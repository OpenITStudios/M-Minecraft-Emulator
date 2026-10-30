// Bounded, ephemeral local window-frame sampling owned by ONE MCP stdio process.
// Frames are collected locally and delivered to Codex/Claude only when asked.
// This is NOT live 30 FPS video, semantic gameplay perception, or game input.
// Game capture, provider latency and Minecraft Windows behavior NOT_EXECUTED.
export const OBSERVATION_LIMITS=Object.freeze({
  minimumIntervalMs:400,maximumIntervalMs:5000,defaultIntervalMs:1250,
  minimumDurationMs:5000,maximumDurationMs:90000,defaultDurationMs:30000,
  maximumSlots:5,maximumEncodedFrameChars:4*1024*1024,
});
const HASH=/^[a-f0-9]{64}$/;
const MAX_ERRORS=3;
const MAX_READ_FRAME_AGE_MS=3500; // Old caches cannot masquerade as recent images.

function allowedSlots(slots) {
  if (!Array.isArray(slots) || slots.length===0 || slots.length>OBSERVATION_LIMITS.maximumSlots ||
      slots.some(slot=>!Number.isInteger(slot) || slot<1 || slot>5) ||
      new Set(slots).size!==slots.length) {
    throw new TypeError('Choose one to five distinct Minecraft slots');
  }
  return [...slots].sort((a,b)=>a-b);
}
function validatedRange(value,min,max,name) {
  if (!Number.isInteger(value) || value<min || value>max) {
    throw new TypeError(name+' is outside its bounded range');
  }
  return value;
}
function frameIdentity(meta) {
  const value=meta?.processIdentity;
  if (!value || !Number.isSafeInteger(value.pid) || value.pid<1 ||
      !Number.isSafeInteger(value.windowId) || value.windowId<1 ||
      !Number.isSafeInteger(value.sessionId) || value.sessionId<0 ||
      typeof value.createdFileTime!=='string' ||
      !/^\d{1,20}$/.test(value.createdFileTime)) {
    throw new TypeError('Capture lacks verified native identity');
  }
  return [value.pid,value.windowId,value.sessionId,value.createdFileTime].join(':');
}
function boundedFrame(frame) {
  const meta=frame?.metadata;
  if (!meta || !['CUA_DRIVER_WINDOW_CAPTURE','NATIVE_WGC_PERSISTENT']
      .includes(meta.provider) ||
      typeof frame.imageBase64!=='string' ||
      frame.imageBase64.length<32 ||
      frame.imageBase64.length>OBSERVATION_LIMITS.maximumEncodedFrameChars ||
      !HASH.test(meta.sha256??'') ||
      !Number.isInteger(meta.width) || meta.width<1 || meta.width>1536 ||
      !Number.isInteger(meta.height) || meta.height<1 || meta.height>1536 ||
      meta.screenshotStatus!=='PNG_RETURNED_NOT_GAMEPLAY_PROOF') {
    throw new TypeError('Unusable frame or unverified capture metadata');
  }
  return {meta,identity:frameIdentity(meta)};
}

/**
 * No external worker, timer service, disk frame store, browser or extra native
 * controller. The caller owns this sampler for the MCP connection lifetime.
 * capture(slot) must return one *already revalidated* observe_client frame.
 */
export function createVisualObserver({
  capture,
  clock=()=>Date.now(),
  schedule=(task,ms)=>setTimeout(task,ms),
  cancel=clearTimeout,
  sourceLabel='BOUNDED_CUA_WINDOW_SNAPSHOTS',
  onFinish=()=>{},
}={}) {
  if (typeof capture!=='function' || typeof onFinish!=='function') {
    throw new TypeError('A verified capture and cleanup callback are required');
  }
  const sessions=new Map();
  let sampling=false;
  let closed=false;

  function clearTimer(state) {
    if (state.timer!==null) {
      cancel(state.timer);
      state.timer=null;
    }
  }
  function finish(state,status) {
    if (!state.active) return;
    state.active=false;
    clearTimer(state);
    state.status=status;
    state.imageBase64=null;
    state.lastHash=null;
    state.identity=null;
    state.nextDueAt=null;
    // A native GPU producer is a separate read-only child. Revoke it as soon
    // as the corresponding observer expires, loses HWND, or becomes unhealthy.
    try {onFinish(state.slot,status);} catch {/* never keep stale pixels */ }
    // Keep bounded counters (not pixels), even when stopped/expired.
  }
  function arm(state,delay) {
    if (!state.active || closed || sessions.get(state.slot)!==state) return;
    clearTimer(state);
    const remaining=state.endsAt-clock();
    if (remaining<=0) {
      finish(state,'EXPIRED');
      return;
    }
    const actual=Math.min(delay,remaining);
    state.nextDueAt=clock()+actual;
    state.timer=schedule(()=>sample(state),actual);
    state.timer?.unref?.();
  }
  async function sample(state) {
    state.timer=null;
    if (closed || !state.active || sessions.get(state.slot)!==state) return;
    if (clock()>=state.endsAt) {
      finish(state,'EXPIRED');
      return;
    }
    if (sampling) {
      state.skippedBusy++;
      arm(state,250);
      return;
    }
    sampling=true;
    let delay=state.intervalMs;
    try {
      // Capture comprises native inspect -> per-window read -> native inspect.
      // It must never call desktop capture or send keyboard/mouse input.
      const frame=await capture(state.slot);
      if (!state.active || sessions.get(state.slot)!==state || closed) return;
      if (clock()>=state.endsAt) {
        finish(state,'EXPIRED');
        return;
      }
      const {meta,identity}=boundedFrame(frame);
      if (state.identity!==null && state.identity!==identity) {
        finish(state,'TARGET_REPLACED_REFUSE_CAPTURE');
        return;
      }
      state.identity=identity;
      state.successfulSamples++;
      state.consecutiveErrors=0;
      state.lastError=null;
      state.lastCollectedMs=clock();
      state.lastCollectedAt=new Date(state.lastCollectedMs).toISOString();
      state.windowBackground=meta.background===true;
      if (meta.sha256!==state.lastHash) {
        state.lastHash=meta.sha256;
        state.sequence++;
        state.imageBase64=frame.imageBase64;
        state.width=meta.width;
        state.height=meta.height;
        state.sameBytesCount=0;
        state.status='FRAME_BYTES_CHANGED_NOT_SEMANTIC_PROOF';
      } else {
        state.sameBytesCount++;
        // PNG byte hashes are NOT pixel-level comparisons, and a still image
        // is not sufficient proof that a game has stopped.
        state.status=state.sameBytesCount>=3 ?
          'STATIC_PNG_BYTES_GAME_STATE_UNKNOWN':'FRAME_BYTES_UNCHANGED';
        delay=Math.min(OBSERVATION_LIMITS.maximumIntervalMs,
          state.intervalMs*(state.sameBytesCount>=3?3:2));
      }
    } catch(error) {
      if (!state.active || sessions.get(state.slot)!==state || closed) return;
      if (/GPU_TARGET_IDENTITY_CHANGED|Minecraft window changed during capture/.test(
          error?.message??'')) {
        state.totalErrors++;
        state.lastError='WINDOW_IDENTITY_UNCONFIRMED';
        finish(state,'TARGET_REPLACED_REFUSE_CAPTURE');
        return;
      }
      if (/in progress|NATIVE_BUSY/.test(error?.message??'')) {
        state.skippedBusy++;
        delay=500;
      } else {
        state.consecutiveErrors++;
        state.totalErrors++;
        state.lastError=/changed|target|identity|stale/i.test(error?.message??'') ?
          'WINDOW_IDENTITY_UNCONFIRMED':'CAPTURE_UNAVAILABLE';
        state.imageBase64=null; // Never serve an old image as a fresh one.
        state.lastHash=null;
        state.status='CAPTURE_ERROR';
        if (state.consecutiveErrors>=MAX_ERRORS) {
          finish(state,'CAPTURE_STOPPED_AFTER_REPEATED_ERRORS');
          return;
        }
        delay=Math.min(OBSERVATION_LIMITS.maximumIntervalMs,state.intervalMs*2);
      }
    } finally {
      sampling=false;
      if (state.active && sessions.get(state.slot)===state && !closed) {
        arm(state,delay);
      }
    }
  }

  function start(slots,{
    intervalMs=OBSERVATION_LIMITS.defaultIntervalMs,
    durationMs=OBSERVATION_LIMITS.defaultDurationMs,
  }={}) {
    if (closed) throw new Error('Visual observation MCP connection is closed');
    const selected=allowedSlots(slots);
    validatedRange(intervalMs,OBSERVATION_LIMITS.minimumIntervalMs,
      OBSERVATION_LIMITS.maximumIntervalMs,'Sampling interval');
    validatedRange(durationMs,OBSERVATION_LIMITS.minimumDurationMs,
      OBSERVATION_LIMITS.maximumDurationMs,'Observation duration');
    for (const slot of selected) {
      const current=sessions.get(slot);
      if (current?.active) throw new Error('Slot already has an active observation');
    }
    const startedAt=clock();
    for (const [index,slot] of selected.entries()) {
      const state={
        slot,active:true,status:'WAITING_FOR_FIRST_FRAME',sequence:0,
        startedAt,endsAt:startedAt+durationMs,intervalMs,
        timer:null,nextDueAt:null,imageBase64:null,
        identity:null,lastHash:null,lastCollectedAt:null,lastCollectedMs:null,
        width:null,height:null,windowBackground:null,
        successfulSamples:0,sameBytesCount:0,
        skippedBusy:0,totalErrors:0,consecutiveErrors:0,lastError:null,
      };
      sessions.set(slot,state);
      arm(state,index*200);
    }
    return {
      schema:1,slots:selected,intervalMs,durationMs,
      status:'LOCAL_OBSERVATION_REQUESTED_NOT_YET_VERIFIED',
      delivery:'ON_DEMAND_CHANGED_FRAMES_ONLY',
      localSampler:sourceLabel,
      liveVideo:false,modelAnalyzesEveryFrame:false,
      gameControl:'NOT_IMPLEMENTED',
    };
  }
  function read(slot,{afterSequence=0,includeImage=true}={}) {
    if (!Number.isInteger(slot) || slot<1 || slot>5) {
      throw new TypeError('Unknown Minecraft slot');
    }
    validatedRange(afterSequence,0,1_000_000_000,'Frame sequence');
    if (typeof includeImage!=='boolean') throw new TypeError('includeImage must be boolean');
    const state=sessions.get(slot);
    if (!state) return {
      metadata:{schema:1,slot,status:'NOT_OBSERVING',
        sequence:0,hasNewImage:false,gameInput:'UNVERIFIED'},
      imageBase64:null,
    };
    if (state.active && clock()>=state.endsAt) finish(state,'EXPIRED');
    const ageMs=state.lastCollectedMs===null ? null :
      Math.max(0,clock()-state.lastCollectedMs);
    const tooOld=ageMs===null || ageMs>MAX_READ_FRAME_AGE_MS;
    const deliver=includeImage && state.active && !tooOld &&
      state.imageBase64!==null && state.sequence>afterSequence;
    return {
      metadata:{
        schema:1,slot,sequence:state.sequence,
        status:state.active && tooOld && state.imageBase64!==null ?
          'CACHED_FRAME_AGED_OUT' : state.status,
        hasNewImage:deliver,active:state.active,
        frameAgeMs:ageMs,maximumReadableFrameAgeMs:MAX_READ_FRAME_AGE_MS,
        intervalMs:state.intervalMs,
        startedAt:new Date(state.startedAt).toISOString(),
        endsAt:new Date(state.endsAt).toISOString(),
        collectedAt:state.lastCollectedAt,
        windowBackground:state.windowBackground,
        width:state.width,height:state.height,
        successfulSamples:state.successfulSamples,
        repeatedPngBytes:state.sameBytesCount,
        skippedBusy:state.skippedBusy,
        captureErrors:state.totalErrors,
        errorCode:state.lastError,
        visualChange:'PNG_BYTE_HASH_ONLY_NOT_SEMANTIC_MOTION',
        modelDelivery:'ONLY_ON_MCP_READ',
        gameInput:'INDEPENDENT_BACKGROUND_INPUT_NOT_VERIFIED',
      },
      imageBase64:deliver?state.imageBase64:null,
    };
  }
  function stop(slots) {
    const selected=allowedSlots(slots);
    for (const slot of selected) {
      const state=sessions.get(slot);
      if (state?.active) finish(state,'STOPPED_BY_REQUEST');
    }
    return {schema:1,slots:selected,status:'STOP_REQUESTED',
      captureFramesRetained:false,gameInput:'NOT_IMPLEMENTED'};
  }
  function shutdown() {
    closed=true;
    for (const state of sessions.values()) finish(state,'CONNECTION_CLOSED');
    sessions.clear();
  }
  return {start,read,stop,shutdown};
}
