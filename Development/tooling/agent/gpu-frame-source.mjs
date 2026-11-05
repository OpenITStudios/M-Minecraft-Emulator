// Optional persistent Windows Graphics Capture stream consumer.
// One bounded process/frame cache per explicit AI-owned Minecraft slot.
// Does NOT launch Minecraft, inject inputs, persist pixels or expose a port.
// Source-only candidate; no local/source/Windows tests authorized yet.
import {spawn} from 'node:child_process';
import {lstat} from 'node:fs/promises';
import path from 'node:path';
import {inspectPng,MAX_PNG_BYTES} from './visual-capture.mjs';

const MAGIC='LTG1', HEADER=28;
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
// A frame is exposed only after a complete PNG and ordered identity-free
// packet header has arrived. The owning worker separately checks HWND.
export function createGpuPacketDecoder(onFrame,{clock=()=>Date.now()}={}) {
  if (typeof onFrame!=='function') throw new TypeError('A frame consumer is required');
  // Assemble one bounded PNG without copying all previously received chunks.
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
        if(header.toString('ascii',0,4)!==MAGIC) {
          throw new Error('Unknown GPU frame protocol');
        }
        const length=header.readUInt32BE(4);
        const width=header.readUInt32BE(8),height=header.readUInt32BE(12);
        const sequence=header.readUInt32BE(16);
        const epoch=header.readBigUInt64BE(20);
        if(length<33 || length>MAX_FRAME ||
           width<1 || height<1 || width>1536 || height>1536 ||
           sequence<=lastSequence || sequence>1000000 ||
           epoch>BigInt(Number.MAX_SAFE_INTEGER)) {
          throw new Error('Invalid GPU frame sequence/size/time');
        }
        const capturedAtMs=Number(epoch);
        // Reject untrusted stale/future header times BEFORE allocating or
        // copying up to 3 MiB of PNG data for a frame we cannot use.
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
      const image=inspectPng(bytes);
      if(image.width!==width || image.height!==height ||
         bytes.toString('ascii',bytes.length-8,bytes.length-4)!=='IEND') {
        throw new Error('GPU frame PNG and transport dimensions disagree');
      }
      const frame={sequence,capturedAtMs,width,height,
        sha256:image.sha256,bytes};
      lastSequence=sequence;
      bytes=null;
      bytesUsed=0;
      fields=null;
      onFrame(frame);
    }
  }
  return {push};
}

/** One provider for an existing MCP process, not a second game session owner. */
export function createGpuFrameSource({
  executable,
  platform=process.platform,
  spawnProcess=spawn,
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
  let closed=false;
  function drop(state,status) {
    if (!state.active) return;
    state.active=false;
    state.status=status;
    state.latest=null;
    if (state.deadlineTimer!==null) {
      cancel(state.deadlineTimer);state.deadlineTimer=null;
    }
    try {state.process.kill();} catch { /* exit is not authority over games */ }
  }
  async function start(slot,target,{durationMs=30000,fps=5}={}) {
    if (closed) throw new Error('GPU capture source closed');
    if (platform!=='win32') throw new Error('GPU capture is Windows-only');
    if (!Number.isInteger(slot) || slot<1 || slot>MAX_SLOTS ||
        !Number.isInteger(durationMs) || durationMs<5000 || durationMs>90000 ||
        !Number.isInteger(fps) || fps<1 || fps>10) {
      throw new TypeError('GPU capture requires one slot, 5-90s and 1-10 fps');
    }
    const selected=validateGpuTarget(target);
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
        '--pid',String(selected.pid),'--window-id',String(selected.windowId),
        '--birth',selected.createdFileTime,'--session',String(selected.sessionId),
        '--duration-ms',String(durationMs),'--fps',String(fps)
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
        const decoder=createGpuPacketDecoder(frame=>{
          if (!state.active) return;
          // A new worker sequence can carry the EXACT same PNG bytes.
          // Reuse the last encoded representation when it already exists,
          // without re-encoding or retaining a second raw PNG buffer.
          const prior=state.latest;
          if(typeof prior?.imageBase64==='string' &&
             prior.sha256===frame.sha256 &&
             prior.width===frame.width && prior.height===frame.height) {
            frame.imageBase64=prior.imageBase64;
            frame.bytes=null;
          }
          state.latest=frame; // still replace sequence/time with new receipt
          state.lastFrameAt=clock();
          state.frames++;
          state.status='FRAME_FROM_NATIVE_WGC_NOT_GAMEPLAY_VERIFIED';
        },{clock});
        child.stdout.on('data',chunk=>{
          if (!state.active) return;
          try {decoder.push(chunk);}
          catch {state.errors++;drop(state,'CORRUPT_GPU_FRAME_STREAM');}
        });
        child.stdout.on('error',()=>drop(state,'GPU_FRAME_STREAM_IO_ERROR'));
        child.on('error',()=>drop(state,'GPU_WORKER_UNAVAILABLE'));
        child.on('close',()=>{
          if(state.active) drop(state,'GPU_WORKER_EXITED');
        });
        state.deadlineTimer=schedule(()=>drop(state,'EXPIRED'),durationMs+2000);
        state.deadlineTimer?.unref?.();
      } catch(error) {
        drop(state,'GPU_START_FAILED');
        throw error;
      }
      return {slot,status:'GPU_SOURCE_STARTED_NO_FRAME_PROOF',
        provider:'NATIVE_WGC_PERSISTENT',durationMs,fps,
        gameInput:'NOT_IMPLEMENTED'};
    } finally {
      if (pendingStarts.get(slot)===reservation) pendingStarts.delete(slot);
    }
  }
  function latest(slot,expectedTarget) {
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
    // Encode once on first demand; release raw bytes and reuse the string.
    // The target identity and source-age checks above still run on every read.
    if(frame.bytes!==null) {
      frame.imageBase64=frame.bytes.toString('base64');
      frame.bytes=null;
    }
    return {
      provider:'NATIVE_WGC_PERSISTENT',
      mimeType:'image/png',imageBase64:frame.imageBase64,
      width:frame.width,height:frame.height,sha256:frame.sha256,
      capturedAtMs:frame.capturedAtMs,
      workerSequence:frame.sequence,
      frameFreshness:'CAPTURE_TIMESTAMP_RECENT_GAMEPLAY_NOT_VERIFIED',
    };
  }
  function stop(slotsToStop) {
    for(const slot of slotsToStop) {
      const reservation=pendingStarts.get(slot);
      if(reservation) reservation.cancelled=true;
      const state=slots.get(slot);
      if(state) drop(state,'STOPPED_BY_REQUEST');
    }
  }
  function shutdown() {
    closed=true;
    for(const reservation of pendingStarts.values()) {
      reservation.cancelled=true;
    }
    stop([...slots.keys()]);
    pendingStarts.clear();
    slots.clear();
  }
  function status(slot) {
    const s=slots.get(slot);
    return s?{slot,status:s.status,frames:s.frames,errors:s.errors,
      active:s.active}:{slot,status:'NOT_STARTED',active:false};
  }
  return {start,latest,stop,shutdown,status};
}
