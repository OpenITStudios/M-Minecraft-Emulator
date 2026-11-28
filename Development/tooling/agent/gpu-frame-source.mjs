// Optional persistent Windows Graphics Capture stream consumer.
// One bounded process/frame cache per explicit AI-owned Minecraft slot.
// Does NOT launch Minecraft, inject inputs, persist pixels or expose a port.
// Source-only candidate; no local/source/Windows tests authorized yet.
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {lstat} from 'node:fs/promises';
import path from 'node:path';
import {inspectPng,MAX_PNG_BYTES} from './visual-capture.mjs';

const MAGIC='LTG1', WEBP_MAGIC='LTG2', HEADER=28;
const execFileAsync=promisify(execFile);
export const DEFAULT_PREVIEW_SIZE=Object.freeze({width:640,height:360});
export function previewSize(width=DEFAULT_PREVIEW_SIZE.width,height=DEFAULT_PREVIEW_SIZE.height) {
  if(!Number.isInteger(width)||!Number.isInteger(height)||width<160||height<160||width>1536||height>1536) {
    throw new Error('Invalid tool arguments');
  }
  return {width,height};
}
// Same restricted RIFF/VP8L structure as the desktop receipt; no new codec.
export function inspectWebp(bytes) {
  if(!Buffer.isBuffer(bytes)||bytes.length<26||bytes.length>MAX_PNG_BYTES||
     bytes.toString('ascii',0,4)!=='RIFF'||bytes.toString('ascii',8,16)!=='WEBPVP8L'||bytes[20]!==0x2f||
     bytes.readUInt32LE(4)+8!==bytes.length)throw new Error('Invalid bounded WebP frame');
  const chunk=bytes.readUInt32LE(16),head=bytes.readUInt32LE(21);
  if(chunk<6||20+chunk+chunk%2!==bytes.length||(chunk%2&&bytes.at(-1)!==0)||head>>>29!==0) {
    throw new Error('Invalid bounded WebP frame');
  }
  const width=(head&0x3fff)+1,height=((head>>>14)&0x3fff)+1;
  if(width>1536||height>1536)throw new Error('Invalid bounded WebP dimensions');
  return {width,height,sha256:createHash('sha256').update(bytes).digest('hex')};
}
const MAX_FRAME=MAX_PNG_BYTES;
const MAX_PENDING=2*MAX_FRAME+HEADER*2;
const MAX_AGE_MS=3500;
const NO_FRAME_TIMEOUT_MS=8000;
const MAX_SLOTS=5;

export function validateGpuExecutable(value) {
  if (typeof value!=='string' || !path.isAbsolute(value) ||
      path.basename(value).toLowerCase()!=='lazy-tester-gpu-capture.exe') {
    throw new TypeError('GPU capture requires an absolute lazy-tester-gpu-capture.exe path');
  }
  return value;
}
export function validateGpuTarget(target) {
  if (!target || !Number.isSafeInteger(target.pid) || target.pid<1 ||
      !Number.isSafeInteger(target.windowId) || target.windowId<1 ||
      !Number.isSafeInteger(target.sessionId) || target.sessionId<0 ||
      typeof target.createdFileTime!=='string' ||
      !/^\d{1,20}$/.test(target.createdFileTime)) {
    throw new TypeError('Native-owned PID/HWND/birth/session target required');
  }
  return {pid:target.pid,windowId:target.windowId,
    createdFileTime:target.createdFileTime,sessionId:target.sessionId};
}

// Parse a binary stdout frame protocol, not arbitrary user-supplied JSON.
// A frame is exposed only after a complete configured image and ordered identity-free
// packet header has arrived. The owning worker separately checks HWND.
export function createGpuPacketDecoder(onFrame,{clock=()=>Date.now(),webpSize=null}={}) {
  if (typeof onFrame!=='function') throw new TypeError('A frame consumer is required');
  if(webpSize!==null)webpSize=previewSize(webpSize.width,webpSize.height);
  // Assemble one bounded image without copying all previously received chunks.
  const header=Buffer.alloc(HEADER);
  let headerUsed=0,bytes=null,bytesUsed=0,fields=null,lastSequence=0;
  function push(chunk) {
    if (!Buffer.isBuffer(chunk) || chunk.length>MAX_PENDING) {
      throw new Error('GPU frame IPC exceeded its bounded buffer');
    }
    let offset=0;
    while(offset<chunk.length) {
      if(bytes===null) {
        const count=Math.min(HEADER-headerUsed,chunk.length-offset);
        chunk.copy(header,headerUsed,offset,offset+count);
        offset+=count;
        headerUsed+=count;
        if(headerUsed<HEADER) break;
        headerUsed=0;
        if(header.toString('ascii',0,4)!==(webpSize?WEBP_MAGIC:MAGIC)) {
          throw new Error('Unknown GPU frame protocol');
        }
        const length=header.readUInt32BE(4);
        const width=header.readUInt32BE(8),height=header.readUInt32BE(12);
        const sequence=header.readUInt32BE(16);
        const epoch=header.readBigUInt64BE(20);
        if(length<(webpSize?26:33) || length>MAX_FRAME ||
           width<1 || height<1 || width>1536 || height>1536 ||
           (webpSize&&(width!==webpSize.width||height!==webpSize.height)) ||
           sequence<=lastSequence || sequence>1000000 ||
           epoch>BigInt(Number.MAX_SAFE_INTEGER)) {
          throw new Error('Invalid GPU frame sequence/size/time');
        }
        const capturedAtMs=Number(epoch);
        // Reject untrusted stale/future header times BEFORE allocating or
        // copying up to 3 MiB of image data for a frame we cannot use.
        const now=clock();
        if(capturedAtMs>now+5000 || capturedAtMs<now-90000) {
          throw new Error('GPU frame timestamp outside capture lifecycle');
        }
        fields={width,height,sequence,capturedAtMs};
        bytes=Buffer.allocUnsafe(length);
        bytesUsed=0;
      }
      const count=Math.min(bytes.length-bytesUsed,chunk.length-offset);
      chunk.copy(bytes,bytesUsed,offset,offset+count);
      offset+=count;
      bytesUsed+=count;
      if(bytesUsed<bytes.length) break;
      const {width,height,sequence,capturedAtMs}=fields;
      // Still recheck after fragmented transfer: time may advance while
      // an otherwise acceptable PNG is being assembled.
      const now=clock();
      if(capturedAtMs>now+5000 || capturedAtMs<now-90000) {
        throw new Error('GPU frame timestamp outside capture lifecycle');
      }
      const image=webpSize?inspectWebp(bytes):inspectPng(bytes);
      if(image.width!==width || image.height!==height ||
         (!webpSize&&bytes.toString('ascii',bytes.length-8,bytes.length-4)!=='IEND')) {
        throw new Error('GPU image and transport dimensions disagree');
      }
      const frame={sequence,capturedAtMs,width,height,
        sha256:image.sha256,bytes,...(webpSize?{mimeType:'image/webp'}:{})};
      lastSequence=sequence;
      bytes=null;
      bytesUsed=0;
      fields=null;
      onFrame(frame);
    }
  }
  return {push,finish:()=>{if(headerUsed!==0||bytes!==null)throw new Error('Incomplete GPU frame stream');}};
}

// Shared image payload: retain raw OR Base64, never both after delivery.
function deferredImage(bytes) {
  let encoded=null;
  const length=4*Math.ceil(bytes.length/3);
  return Object.freeze({length,read(){
    if(bytes!==null) {
      encoded=bytes.toString('base64');
      bytes=null;
    }
    return encoded;
  }});
}

/** One provider for an existing operation core, not a second game session owner. */
export function createGpuFrameSource({
  executable,
  platform=process.platform,
  spawnProcess=spawn,
  executePreview=execFileAsync,
  statFile=lstat,
  clock=()=>Date.now(),
  schedule=setTimeout,
  cancel=clearTimeout,
}={}) {
  validateGpuExecutable(executable);
  const slots=new Map();
  // Reserve each slot before the asynchronous file check. Otherwise a
  // concurrent start/stop or connection shutdown can leave an orphan worker.
  const pendingStarts=new Map();
  // A stop request is not exit proof. Retain only the per-slot worker
  // reference until its close event, preventing replacement overlap.
  const stopping=new Map();
  let closed=false;
  function drop(state,status,exited=false) {
    if (!state.active) return;
    state.active=false;
    state.status=status;
    state.latest=null;
    if (state.deadlineTimer!==null) {
      cancel(state.deadlineTimer);state.deadlineTimer=null;
    }
    if(!exited)stopping.set(state.slot,state);
    // The source's slot map must not permanently retain a stopped child,
    // stdout listeners, decoder or partially assembled 3 MiB PNG.
    // Preserve only the existing bounded status counters for diagnostics.
    // Late callbacks still see their own inactive state and cannot replace
    // or revoke a newly started worker on this same slot.
    if (slots.get(state.slot)===state) {
      slots.set(state.slot,{slot:state.slot,status,frames:state.frames,
        errors:state.errors,active:false});
    }
    if(!exited)try {state.process.kill();} catch { /* keep slot reserved until close */ }
  }
  async function start(slot,target,{durationMs=30000,fps=5,webpSize=null}={}) {
    if(webpSize!==null)webpSize=previewSize(webpSize.width,webpSize.height);
    if (closed) throw new Error('GPU capture source closed');
    if (platform!=='win32') throw new Error('GPU capture is Windows-only');
    if (!Number.isInteger(slot) || slot<1 || slot>MAX_SLOTS ||
        !Number.isInteger(durationMs) || durationMs<5000 || durationMs>90000 ||
        !Number.isInteger(fps) || fps<1 || fps>10) {
      throw new TypeError('GPU capture requires one slot, 5-90s and 1-10 fps');
    }
    const selected=validateGpuTarget(target);
    if(stopping.has(slot))throw new Error('GPU_WORKER_STOP_PENDING');
    if (slots.get(slot)?.active || pendingStarts.has(slot)) {
      throw new Error('GPU source already active or starting on this slot');
    }
    const reservation={cancelled:false};
    pendingStarts.set(slot,reservation);
    try {
      const file=await statFile(executable);
      if (closed || reservation.cancelled) {
        throw new Error('GPU_SOURCE_START_CANCELLED');
      }
      if (!file.isFile() || file.isSymbolicLink()) {
        throw new Error('GPU worker executable must be an ordinary file');
      }
      const args=[
        ...(webpSize?['--webp']:[]),
        '--pid',String(selected.pid),'--window-id',String(selected.windowId),
        '--birth',selected.createdFileTime,'--session',String(selected.sessionId),
        '--duration-ms',String(durationMs),'--fps',String(fps),
        ...(webpSize?['--width',String(webpSize.width),'--height',String(webpSize.height)]:[])
      ];
      const child=spawnProcess(executable,args,
        {shell:false,windowsHide:true,stdio:['ignore','pipe','ignore']});
      if (!child?.stdout?.on || typeof child.on!=='function' ||
          typeof child.kill!=='function') {
        // A worker that cannot be supervised must not be left running.
        try {child?.kill?.();} catch { /* retain a hard startup refusal */ }
        throw new Error('GPU worker did not open a bounded stdout pipe');
      }
      const state={
        slot,identity:selected,process:child,active:true,status:'STARTING',
        latest:null,lastFrameAt:clock(),startedAt:clock(),
        deadlineTimer:null,errors:0,frames:0,
      };
      slots.set(slot,state);
      try {
        child.on('close',()=>{
          if(state.active)drop(state,'GPU_WORKER_EXITED',true);
          if(stopping.get(slot)===state)stopping.delete(slot);
        });
        const decoder=createGpuPacketDecoder(frame=>{
          if (!state.active) return;
          // Producer and observer share a payload for identical bytes,
          // while independently advancing verified frame receipts.
          const prior=state.latest;
          const {bytes,...receipt}=frame;
          const imageRef=prior&&prior.sha256===frame.sha256&&
            prior.mimeType===frame.mimeType&&prior.width===frame.width&&
            prior.height===frame.height?prior.imageRef:deferredImage(bytes);
          state.latest={...receipt,imageRef};
          state.lastFrameAt=clock();
          state.frames++;
          state.status='FRAME_FROM_NATIVE_WGC_NOT_GAMEPLAY_VERIFIED';
        },{clock,webpSize});
        child.stdout.on('data',chunk=>{
          if (!state.active) return;
          try {decoder.push(chunk);}
          catch {state.errors++;drop(state,'CORRUPT_GPU_FRAME_STREAM');}
        });
        child.stdout.on('error',()=>drop(state,'GPU_FRAME_STREAM_IO_ERROR'));
        child.on('error',()=>drop(state,'GPU_WORKER_UNAVAILABLE'));
        state.deadlineTimer=schedule(()=>drop(state,'EXPIRED'),durationMs+2000);
        state.deadlineTimer?.unref?.();
      } catch(error) {
        drop(state,'GPU_START_FAILED');
        throw error;
      }
      return {slot,status:'GPU_SOURCE_STARTED_NO_FRAME_PROOF',
        provider:'NATIVE_WGC_PERSISTENT',durationMs,fps,
        ...(webpSize?{mimeType:'image/webp',...webpSize}:{}),
        gameInput:'NOT_IMPLEMENTED'};
    } finally {
      if (pendingStarts.get(slot)===reservation) pendingStarts.delete(slot);
    }
  }
  function latest(slot,expectedTarget,{deferImage=false}={}) {
    if (!Number.isInteger(slot) || slot<1 || slot>MAX_SLOTS) {
      throw new TypeError('Unknown Minecraft profile');
    }
    const state=slots.get(slot);
    if (!state?.active) throw new Error('GPU_SOURCE_NOT_ACTIVE');
    // The source is pinned at start to an owned game process. A fresh
    // native target must match that binding at EVERY frame read. Checking
    // native PID/HWND only before and after latest() is insufficient:
    // an old GPU worker can still have a cached PNG for a replaced client.
    const expected=validateGpuTarget(expectedTarget);
    if (expected.pid!==state.identity.pid ||
        expected.windowId!==state.identity.windowId ||
        expected.createdFileTime!==state.identity.createdFileTime ||
        expected.sessionId!==state.identity.sessionId) {
      drop(state,'TARGET_IDENTITY_CHANGED'); // revokes worker and old PNG
      throw new Error('GPU_TARGET_IDENTITY_CHANGED');
    }
    const now=clock();
    if (now-state.lastFrameAt>NO_FRAME_TIMEOUT_MS) {
      drop(state,'GPU_STREAM_STALLED');
      throw new Error('GPU_STREAM_STALLED');
    }
    const frame=state.latest;
    // A future-dated transport frame is not evidence of a fresh capture.
    if (!frame || frame.capturedAtMs>now ||
        now-frame.capturedAtMs>MAX_AGE_MS) {
      throw new Error('GPU_FRAME_NOT_READY');
    }
    // Only the internal sampler requests the private image reference.
    return {
      provider:'NATIVE_WGC_PERSISTENT',
      mimeType:frame.mimeType??'image/png',
      ...(deferImage?{imageRef:frame.imageRef}:{imageBase64:frame.imageRef.read()}),
      width:frame.width,height:frame.height,sha256:frame.sha256,
      capturedAtMs:frame.capturedAtMs,
      workerSequence:frame.sequence,
      frameFreshness:'CAPTURE_TIMESTAMP_RECENT_GAMEPLAY_NOT_VERIFIED',
    };
  }
  // One-shot uses the SAME worker and target validation, without an
  // active sampler or a second capture process for the same slot.
  async function captureOnce(slot,target,size=DEFAULT_PREVIEW_SIZE) {
    if(closed||platform!=='win32')throw new Error('GPU_PREVIEW_UNAVAILABLE');
    if(!Number.isInteger(slot)||slot<1||slot>MAX_SLOTS)throw new Error('Invalid tool arguments');
    const webpSize=previewSize(size.width,size.height);
    const selected=validateGpuTarget(target);
    if(stopping.has(slot))throw new Error('GPU_WORKER_STOP_PENDING');
    if(slots.get(slot)?.active||pendingStarts.has(slot))throw new Error('GPU_PREVIEW_USE_EXISTING_SAMPLER');
    const abort=new AbortController();
    const reservation={cancelled:false,abort};
    pendingStarts.set(slot,reservation);
    let waitingForClose=false;
    try {
      const file=await statFile(executable);
      if(closed||reservation.cancelled||!file.isFile()||file.isSymbolicLink()) {
        throw new Error('GPU_PREVIEW_UNAVAILABLE');
      }
      const args=['--preview','--pid',String(selected.pid),'--window-id',String(selected.windowId),
        '--birth',selected.createdFileTime,'--session',String(selected.sessionId),
        '--width',String(webpSize.width),'--height',String(webpSize.height)];
      let output;
      try {
        const execution=executePreview(executable,args,{
          encoding:'buffer',shell:false,windowsHide:true,timeout:8000,
          maxBuffer:MAX_FRAME+HEADER,signal:abort.signal});
        // Promisified execFile exposes its exact ChildProcess. Abort can
        // reject before close; neither a rejected promise nor kill proves exit.
        waitingForClose=true;
        if(typeof execution?.child?.once!=='function') {
          void Promise.resolve(execution).catch(()=>{});
          abort.abort();
          throw new Error('GPU_PREVIEW_UNAVAILABLE');
        }
        execution.child.once('close',()=>{
          waitingForClose=false;
          if(stopping.get(slot)===reservation)stopping.delete(slot);
        });
        ({stdout:output}=await execution);
      }catch {throw new Error('GPU_PREVIEW_UNAVAILABLE');}
      if(waitingForClose||closed||reservation.cancelled||!Buffer.isBuffer(output)||output.length>MAX_FRAME+HEADER) {
        throw new Error('GPU_PREVIEW_UNAVAILABLE');
      }
      let frame=null;
      try {
        const decoder=createGpuPacketDecoder(value=>{
          if(frame!==null||value.sequence!==1)throw new Error('Expected one preview');
          frame=value;
        },{clock,webpSize});
        decoder.push(output);
        decoder.finish();
      }catch {throw new Error('GPU_PREVIEW_UNAVAILABLE');}
      const now=clock();
      if(!frame||frame.capturedAtMs>now||now-frame.capturedAtMs>MAX_AGE_MS) {
        throw new Error('GPU_PREVIEW_UNAVAILABLE');
      }
      return {provider:'NATIVE_WGC_ON_DEMAND',mimeType:'image/webp',
        bytes:frame.bytes,width:frame.width,height:frame.height,
        sha256:frame.sha256,capturedAtMs:frame.capturedAtMs,workerSequence:frame.sequence,
        frameFreshness:'CAPTURE_TIMESTAMP_RECENT_GAMEPLAY_NOT_VERIFIED'};
    }finally {
      // Do not allow a new worker while the exact preview is still closing.
      if(!closed&&waitingForClose)stopping.set(slot,reservation);
      if(pendingStarts.get(slot)===reservation)pendingStarts.delete(slot);
    }
  }
  function stop(slotsToStop) {
    for(const slot of slotsToStop) {
      const reservation=pendingStarts.get(slot);
      if(reservation) {reservation.cancelled=true;reservation.abort?.abort();}
      const state=slots.get(slot);
      if(state) drop(state,'STOPPED_BY_REQUEST');
    }
  }
  function shutdown() {
    closed=true;
    for(const reservation of pendingStarts.values()) {
      reservation.cancelled=true;reservation.abort?.abort();
    }
    stop([...slots.keys()]);
    pendingStarts.clear();
    slots.clear();
    stopping.clear(); // closed source cannot admit a replacement
  }
  function status(slot) {
    const s=slots.get(slot);
    return s?{slot,status:s.status,frames:s.frames,errors:s.errors,
      active:s.active}:{slot,status:'NOT_STARTED',active:false};
  }
  return {start,latest,captureOnce,stop,shutdown,status};
}
