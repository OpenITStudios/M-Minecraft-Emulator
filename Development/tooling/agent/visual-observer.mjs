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
    state.captureIdentity=null;
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
      // Only source-captured, before/after verified process identifiers.
      // They may be exposed for per-slot consistency checks while a fresh
      // image exists, never as a Microsoft account or world identity.
      state.captureIdentity={
        pid:meta.processIdentity.pid,
        windowId:meta.processIdentity.windowId,
        sessionId:meta.processIdentity.sessionId,
        createdFileTime:meta.processIdentity.createdFileTime,
      };
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
        identity:null,captureIdentity:null,
        lastHash:null,lastCollectedAt:null,lastCollectedMs:null,
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
    const fresh=state.active && !tooOld &&
      state.imageBase64!==null && state.captureIdentity!==null;
    const deliver=includeImage && fresh && state.sequence>afterSequence;
    return {
      metadata:{
        schema:1,slot,sequence:state.sequence,
        status:state.active && tooOld && state.imageBase64!==null ?
          'CACHED_FRAME_AGED_OUT' : state.status,
        hasNewImage:deliver,active:state.active,
        // Frame availability is independent from whether PNG bytes changed
        // or whether this particular MCP read requested a binary image.
        freshFrameAvailable:fresh,
        captureIdentity:fresh?{...state.captureIdentity}:null,
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

// A pure bounded cross-client view over the SAME ephemeral per-slot frame
// cache. This does not collect screenshots, decide game actions, create a
// provider or infer gameplay from changed PNG bytes.
export function summarizeTeamVisualState(mode,slots,readings,since=[]) {
  if(!['human-plus-four-ai','five-ai'].includes(mode)) {
    throw new TypeError('Unknown AI team mode');
  }
  const selected=allowedSlots(slots);
  if(mode==='human-plus-four-ai' && selected.includes(1)) {
    throw new TypeError('Human MC01 cannot be an AI visual target');
  }
  if(!Array.isArray(readings)||readings.length!==selected.length ||
     !Array.isArray(since)||since.length>selected.length) {
    throw new TypeError('Incomplete multi-client visual observations');
  }
  const baseline=new Map();
  for(const entry of since){
    if(!entry||!Number.isInteger(entry.slot)||!selected.includes(entry.slot)||
       baseline.has(entry.slot)||
       !Number.isSafeInteger(entry.sequence)||entry.sequence<0||
       !Number.isSafeInteger(entry.samples)||entry.samples<0) {
      throw new TypeError('Invalid per-client visual baseline');
    }
    baseline.set(entry.slot,entry);
  }
  if(since.length>0 && baseline.size!==selected.length) {
    throw new TypeError('Visual baseline must cover every selected client');
  }
  const seen=new Set();
  const projection=[];
  for(const meta of readings) {
    const slot=meta?.slot;
    if(meta?.schema!==1||!selected.includes(slot)||seen.has(slot)) {
      throw new TypeError('Invalid/duplicate visual client result');
    }
    seen.add(slot);
    const identity=meta.captureIdentity;
    const validIdentity=identity&&Number.isSafeInteger(identity.pid)&&identity.pid>0&&
      Number.isSafeInteger(identity.windowId)&&identity.windowId>0&&
      Number.isSafeInteger(identity.sessionId)&&identity.sessionId>=0&&
      typeof identity.createdFileTime==='string'&&
      /^\d{1,20}$/.test(identity.createdFileTime);
    const fresh=meta.active===true&&meta.freshFrameAvailable===true&&
      validIdentity && Number.isSafeInteger(meta.sequence)&&meta.sequence>0&&
      Number.isSafeInteger(meta.successfulSamples)&&meta.successfulSamples>0&&
      Number.isSafeInteger(meta.frameAgeMs)&&meta.frameAgeMs>=0&&
      meta.frameAgeMs<=3500&&typeof meta.collectedAt==='string'&&
      Number.isFinite(Date.parse(meta.collectedAt));
    const previous=baseline.get(slot);
    const sameTarget=Boolean(previous&&fresh&&
      previous.captureIdentity?.pid===identity.pid&&
      previous.captureIdentity?.windowId===identity.windowId&&
      previous.captureIdentity?.sessionId===identity.sessionId&&
      previous.captureIdentity?.createdFileTime===identity.createdFileTime);
    const sampledSince=previous? Boolean(sameTarget&&
      meta.successfulSamples>previous.samples):null;
    const pngBytesChanged=previous?Boolean(sameTarget&&
      meta.sequence>previous.sequence):null;
    const basis=fresh?{
      slot,sequence:meta.sequence,samples:meta.successfulSamples,
      collectedAt:meta.collectedAt,captureIdentity:{...identity},
    }:null;
    projection.push({
      slot,status:typeof meta.status==='string'?meta.status:'UNAVAILABLE',
      active:meta.active===true,freshFrameAvailable:Boolean(fresh),
      frameAgeMs:Number.isSafeInteger(meta.frameAgeMs)?meta.frameAgeMs:null,
      sequence:Number.isSafeInteger(meta.sequence)?meta.sequence:0,
      samples:Number.isSafeInteger(meta.successfulSamples)?meta.successfulSamples:0,
      collectedAt:typeof meta.collectedAt==='string'?meta.collectedAt:null,
      baseline:basis,observedSameProcessSinceBaseline:previous?sameTarget:null,
      sampledSinceBaseline:sampledSince,pngBytesChangedSinceBaseline:pngBytesChanged,
      // Only requesting a single already-cached frame via existing
      // read_visual_observation passes image bytes to Codex/Claude.
      imageRead:fresh?{tool:'read_visual_observation',slot,afterSequence:0,
        includeImage:true}:null,
      gameplayChange:'NOT_EVALUATED_BY_MINECRAFT_OR_MODEL',
    });
  }
  projection.sort((a,b)=>a.slot-b.slot);
  const allRecent=projection.length===selected.length&&
    projection.every(item=>item.freshFrameAvailable);
  return {
    schema:1,mode,slots:selected,source:'EXISTING_BOUNDED_VISUAL_OBSERVER',
    status:allRecent?'RECENT_PER_SLOT_FRAMES_AVAILABLE':
      'INCOMPLETE_OR_STALE_VISUAL_OBSERVATION',
    allSlotsHaveRecentFrames:allRecent,
    frameCount:projection.filter(item=>item.freshFrameAvailable).length,
    frames:projection,imagesIncluded:false,
    decision:'REQUIRES_EXPLICIT_AI_IMAGE_REVIEW',
    motionInference:'PNG_BYTES_AND_TIMESTAMPS_NOT_GAMEPLAY_PROOF',
    note:'One metadata read of 1-5 existing local caches; no new capture, gameplay input, world identity, or automated AI decision is implied.',
  };
}

// An observed experimental action must be grounded in recent per-client
// cached frames. This is a freshness check, NOT game-input authorization;
// the same retained native controller still verifies each actual write.
// The exact sample/identity prevents inadvertent use of a previous
// window or a newer unseen frame to justify a stale decision.
export function verifyTeamObservationBasis(mode,slots,basis,readings) {
  const current=summarizeTeamVisualState(mode,slots,readings);
  if(!current.allSlotsHaveRecentFrames) {
    throw new Error('TEAM_OBSERVATION_INCOMPLETE');
  }
  if(!Array.isArray(basis)||basis.length!==current.slots.length) {
    throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
  }
  const expected=new Map();
  for(const item of basis) {
    if(!item||!Number.isInteger(item.slot)||expected.has(item.slot)) {
      throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
    }
    expected.set(item.slot,item);
  }
  for(const frame of current.frames) {
    const actual=frame.baseline;
    const passed=expected.get(frame.slot);
    const id=passed?.captureIdentity;
    if(!actual||!passed||
       passed.sequence!==actual.sequence||
       passed.samples!==actual.samples||
       passed.collectedAt!==actual.collectedAt||
       id?.pid!==actual.captureIdentity.pid||
       id?.windowId!==actual.captureIdentity.windowId||
       id?.sessionId!==actual.captureIdentity.sessionId||
       id?.createdFileTime!==actual.captureIdentity.createdFileTime) {
      throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
    }
  }
  return current;
}


// Conservative per-client feedback over a completed *caller-supplied*
// experimental action receipt and the EXISTING bounded visual cache.
// The model must review before/after PNG images separately. A reported
// appearance is NEVER a Minecraft input, hit, movement, or game PASS.
const VISUAL_ASSESSMENTS=new Set([
  'VISIBLE_CHANGE_REPORTED','NO_VISIBLE_CHANGE_REPORTED','UNDETERMINED',
]);
const VISUAL_CATEGORIES=new Set(['movement','camera','attack','interaction']);
function visualCategory(kind) {
  if(['move_forward','move_backward','strafe_left','strafe_right',
      'jump','sneak'].includes(kind)) return 'movement';
  if(kind==='look_relative') return 'camera';
  if(kind==='attack') return 'attack';
  if(kind==='use') return 'interaction';
  return null;
}

/**
 * Evaluates only whether the model's bounded per-slot visual reports can
 * be associated with newer, same-process frames after a caller-reported
 * native action round. The receipt itself is NOT authenticated here:
 * no durable action ledger, external gameplay telemetry or second owner.
 */
export function evaluateTeamActionFeedback(mode,actions,receipt,comparison,after,reviews=[]) {
  if(!['human-plus-four-ai','five-ai'].includes(mode) ||
     !Array.isArray(actions)||actions.length<1||actions.length>32 ||
     !Array.isArray(reviews)||reviews.length>20 ||
     !Array.isArray(after)||after.length>5) {
    throw new TypeError('Invalid bounded multi-client evaluation');
  }
  const requested=new Map();
  for(const action of actions) {
    const kind=visualCategory(action?.kind);
    const slot=action?.slot;
    if(!kind||!Number.isInteger(slot)||slot<1||slot>5||
       (mode==='human-plus-four-ai'&&slot===1)) {
      throw new TypeError('Invalid AI evaluation action or human-owned slot');
    }
    if(!requested.has(slot))requested.set(slot,new Set());
    requested.get(slot).add(kind);
  }
  const slots=[...requested.keys()].sort((a,b)=>a-b);
  const exactly=(list)=>Array.isArray(list)&&list.length===slots.length&&
    list.every((slot,i)=>slot===slots[i]);
  if(receipt?.schema!==1||receipt?.mode!==mode||
     receipt?.status!=='EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED'||
     receipt?.actionResult!=='NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE'||
     receipt?.gameplayObserved!==false||
     !exactly(receipt?.slots)||
     !Array.isArray(receipt?.visualBaseline)||
     receipt.visualBaseline.length!==slots.length ||
     after.length!==slots.length ||
     comparison?.schema!==1||comparison?.mode!==mode||
     !exactly(comparison.slots)||
     !Array.isArray(comparison.frames)||comparison.frames.length!==slots.length) {
    throw new TypeError('Invalid or incompatible unverified action receipt');
  }
  const start=Date.parse(receipt.transportStartedAt);
  const end=Date.parse(receipt.transportCompletedAt);
  if(!Number.isFinite(start)||!Number.isFinite(end)||end<start) {
    throw new TypeError('Action round has no bounded timestamp');
  }
  const first=new Map();
  for(const item of receipt.visualBaseline) {
    if(!item||!slots.includes(item.slot)||first.has(item.slot)||
       !Number.isSafeInteger(item.sequence)||item.sequence<1||
       !Number.isSafeInteger(item.samples)||item.samples<1||
       !Number.isFinite(Date.parse(item.collectedAt)) ||
       !item.captureIdentity) {
      throw new TypeError('Invalid per-slot visual baseline in action receipt');
    }
    first.set(item.slot,item);
  }
  const reported=new Map();
  for(const row of reviews) {
    if(!row||!slots.includes(row.slot)||!VISUAL_CATEGORIES.has(row.category)||
       !requested.get(row.slot).has(row.category)||
       !VISUAL_ASSESSMENTS.has(row.assessment)) {
      throw new TypeError('Invalid model visual report or unrequested category');
    }
    const key=row.slot+':'+row.category;
    if(reported.has(key))throw new TypeError('Duplicate model visual report');
    const cue=row.cue;
    if(row.assessment!=='UNDETERMINED' &&
       (typeof cue!=='string'||cue.trim().length<8||cue.length>160||
        /[\u0000-\u001f\u007f]/.test(cue))) {
      throw new TypeError('Visible finding needs a bounded scene-specific cue');
    }
    if(cue!==undefined&&(typeof cue!=='string'||cue.length>160||
       /[\u0000-\u001f\u007f]/.test(cue))) {
      throw new TypeError('Invalid visual report cue');
    }
    reported.set(key,{assessment:row.assessment,cue:cue?.trim()||null});
  }
  const outcomes=[];
  const totals={reportedVisibleChanges:0,reportedNoVisibleChanges:0,
    undetermined:0,notEvaluable:0};
  for(const [index,slot] of slots.entries()) {
    const frame=comparison.frames[index];
    const before=first.get(slot);
    if(!frame||frame.slot!==slot||!before) {
      throw new TypeError('Missing per-slot comparison');
    }
    const beforeTime=Date.parse(before.collectedAt);
    const afterTime=Date.parse(frame.collectedAt);
    const sameTarget=frame.observedSameProcessSinceBaseline===true;
    const suppliedAfter=after[index]; // may be null for a missing capture
    const current=frame.baseline;
    const afterMatches=Boolean(suppliedAfter&&current&&
      suppliedAfter.slot===slot&&
      suppliedAfter.sequence===current.sequence&&
      suppliedAfter.samples===current.samples&&
      suppliedAfter.collectedAt===current.collectedAt&&
      suppliedAfter.captureIdentity?.pid===current.captureIdentity?.pid&&
      suppliedAfter.captureIdentity?.windowId===current.captureIdentity?.windowId&&
      suppliedAfter.captureIdentity?.sessionId===current.captureIdentity?.sessionId&&
      suppliedAfter.captureIdentity?.createdFileTime===
        current.captureIdentity?.createdFileTime);
    // No accepted post-action observation without a fresh, newly captured
    // frame on the SAME game HWND and an ordering after completion.
    const usable=frame.freshFrameAvailable===true && afterMatches &&
      sameTarget && frame.sampledSinceBaseline===true &&
      Number.isSafeInteger(frame.samples)&&frame.samples>before.samples&&
      Number.isSafeInteger(frame.sequence)&&frame.sequence>=before.sequence&&
      Number.isFinite(beforeTime)&&Number.isFinite(afterTime)&&
      beforeTime<=start&&afterTime>end;
    const cause=!frame.freshFrameAvailable?'NO_RECENT_POST_FRAME':
      !afterMatches?'POST_FRAME_SNAPSHOT_MISSING_OR_CHANGED':
      !sameTarget?'GAME_WINDOW_IDENTITY_CHANGED':
      frame.sampledSinceBaseline!==true?'NO_NEW_SAME_PROCESS_SAMPLE':
      beforeTime>start||!Number.isFinite(afterTime)||afterTime<=end?
        'FRAME_TIMELINE_NOT_AFTER_ACTION':
      frame.sequence<before.sequence?'FRAME_SEQUENCE_REGRESSION':'NONE';
    const categories=[...requested.get(slot)].sort();
    const findings=categories.map(category=>{
      const review=reported.get(slot+':'+category);
      let status='UNDETERMINED_NOT_REVIEWED';
      if(!usable) status='NOT_EVALUABLE';
      else if(review?.assessment==='VISIBLE_CHANGE_REPORTED') {
        status=frame.pngBytesChangedSinceBaseline===true?
          'MODEL_REPORTS_VISUAL_CHANGE_UNVERIFIED':
          'MODEL_REPORT_CONFLICTS_WITH_IDENTICAL_PNG';
      } else if(review?.assessment==='NO_VISIBLE_CHANGE_REPORTED') {
        status='MODEL_REPORTS_NO_VISIBLE_CHANGE_UNVERIFIED';
      } else if(review?.assessment==='UNDETERMINED') {
        status='UNDETERMINED_BY_MODEL';
      }
      if(status==='MODEL_REPORTS_VISUAL_CHANGE_UNVERIFIED')
        totals.reportedVisibleChanges++;
      else if(status==='MODEL_REPORTS_NO_VISIBLE_CHANGE_UNVERIFIED')
        totals.reportedNoVisibleChanges++;
      else if(status==='NOT_EVALUABLE')totals.notEvaluable++;
      else totals.undetermined++;
      return {category,status,cue:usable?(review?.cue??null):null,
        reportedBy:review?'EXTERNAL_MODEL_UNVERIFIED':null,
        gameplayConfirmed:false};
    });
    outcomes.push({
      slot,postFrameUsable:Boolean(usable),
      sourceStatus:usable?'NEWER_SAME_PROCESS_FRAME':
        cause==='NONE'?'INCOMPLETE_FRAME_CONTRACT':cause,
      pngBytesChanged:usable?frame.pngBytesChangedSinceBaseline:null,
      samplesAfterBaseline:usable?frame.samples-before.samples:null,
      findings,
    });
  }
  return {
    schema:1,mode,slots,status:totals.notEvaluable?
      'INCOMPLETE_VISUAL_EVALUATION':
      (totals.reportedVisibleChanges+totals.reportedNoVisibleChanges>0?
        'MODEL_VISUAL_REPORT_ONLY':'UNDETERMINED_NO_VISUAL_REPORT'),
    outcomes,totals,imagesIncluded:false,
    transportReceipt:'CALLER_SUPPLIED_NOT_AUTHENTICATED_BY_EVALUATOR',
    visualReview:'EXTERNAL_MODEL_REPORT_NOT_INDEPENDENTLY_VERIFIED',
    actionTiming:'CALLER_SUPPLIED_TIMESTAMPS_AND_CAPTURE_METADATA_ONLY',
    gameplayAcceptance:'NOT_VERIFIED',
    worldAndPlayerIdentity:'NOT_VERIFIED',
    note:'A visible change is a model-reported appearance, not verified player movement, camera control, an attack hit, or Minecraft input acceptance. A lack of visible change is not proof that a game action failed.',
  };
}
