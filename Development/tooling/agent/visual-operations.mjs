// Owns visual observation operations for one existing operation core.
// No additional native transport, input writer, or independent lifecycle.
import {createHash} from 'node:crypto';
import {slotOf,keysExactly} from './agent-contract.mjs';
import {sameNativeVisualTarget} from './native-transport.mjs';
import {previewSize} from './gpu-frame-source.mjs';
import {OBSERVATION_LIMITS,projectF3VisualReview,reviewVisibleXYZFromFrame} from './visual-observer.mjs';
import {assessCuaStrictBackgroundAction,validateCuaStrictBackgroundIntent} from './visual-capture.mjs';

export function createVisualOperations(config,{
  capture,inspectCua,gpuSourceFactory,visualObserverFactory,readNative,
  mc02Enabled,multiEnabled,requireOpen,isNativeBusy,setNativeBusy,
  hasActiveSequence,revokeObservedRound,callTool,
}) {
  let visualReadBusy=false;
  let gpuStartup=null;
  const exclusiveBusy=()=>isNativeBusy()||visualReadBusy;
  const gpuSource=config.gpuExecutable ?
    gpuSourceFactory({executable:config.gpuExecutable}):null;
  function protectedSlot(slot) {
    if (config.mode==='human-plus-four-ai' && slot===1) {
      throw new Error('MC01 human host is protected from AI visual observation');
    }
  }
  function visualSize(args) {
    if(args.imageFormat===undefined||args.imageFormat==='png') {
      if(args.width!==undefined||args.height!==undefined)throw new Error('Invalid tool arguments');
      return null;
    }
    if(args.imageFormat!=='webp'||mc02Enabled||multiEnabled)throw new Error('Invalid tool arguments');
    if(!gpuSource)throw new Error('WEBP_CAPTURE_REQUIRES_WGC');
    if((args.width===undefined)!==(args.height===undefined))throw new Error('Invalid tool arguments');
    return previewSize(args.width,args.height);
  }
  async function captureVerifiedFrame(slot,onceSize=null,afterFrame=undefined,deferImage=false) {
    protectedSlot(slot);
    if (!config.cuaExecutable && !gpuSource) {
      throw new Error('Visual provider is NOT_CONFIGURED; set --cua-exe or --gpu-capture-exe');
    }
    // A verified read-only capture may accompany an already-running
    // experimental MC02 sequence. It must never clear that sequence's
    // native mutation ownership on completion.
    const alongsideSequence=hasActiveSequence();
    if(visualReadBusy || (isNativeBusy() && !alongsideSequence)) {
      throw new Error('Native operation in progress; do not overlap requests');
    }
    // This read-only gate is sufficient to block new native mutations
    // via exclusiveBusy(). Never claim ownership of inFlight: a separate
    // urgent neutral-state write may overlap a screenshot and must keep
    // its own mutation lock until its authenticated request completes.
    visualReadBusy=true;
    try {
      const before=await readNative('target',slot);
      requireOpen();
      // GPU provider uses a bounded one-shot or an explicitly started
      // persistent worker. No fallback to desktop/global capture.
      // Read ONLY from the WGC worker pinned to this same exact native
      // process/window/birth/session. Without this check a restarted slot
      // could receive its predecessor's cached frame.
      // A Cua screenshot has no trustworthy pixel-presentation timestamp.
      // The query-start time is a conservative lower bound on WHEN a
      // screenshot request began (not when the game rendered its pixels).
      // WGC carries a separate producer-authored readback time instead.
      const captureStartedAtMs=Date.now();
      const frame=gpuSource ? (onceSize?await gpuSource.captureOnce(slot,before,onceSize):gpuSource.latest(slot,before,{deferImage})) :
        await capture(config.cuaExecutable,before);
      requireOpen();
      const after=await readNative('target',slot);
      requireOpen();
      if (!sameNativeVisualTarget(before,after)) {
        throw new Error('Minecraft window changed during capture; screenshot discarded');
      }
      const webp=frame?.mimeType==='image/webp';
      const deferred=deferImage&&gpuSource&&!onceSize&&frame?.imageRef!==undefined;
      if(webp&&(!Number.isSafeInteger(frame.capturedAtMs)||frame.capturedAtMs<1||
         Date.now()-frame.capturedAtMs<0||Date.now()-frame.capturedAtMs>3500)) {
        throw new Error('GPU_PREVIEW_UNAVAILABLE');
      }
      if (!frame || (!webp&&frame.mimeType!=='image/png') ||
          (webp&&(!gpuSource||mc02Enabled||multiEnabled)) ||
          (onceSize&&(!webp||frame.width!==onceSize.width||frame.height!==onceSize.height)) ||
          (!onceSize&&!deferred&&typeof frame.imageBase64!=='string') ||
          (onceSize&&(Buffer.isBuffer(frame.bytes)?
            frame.bytes.length<1||frame.bytes.length>3*1024*1024:
            typeof frame.imageBase64!=='string')) ||
          !Number.isInteger(frame.width) || !Number.isInteger(frame.height) ||
          frame.width<1 || frame.height<1) {
        throw new Error('Visual provider did not produce a usable frame');
      }
      const observation = {
        metadata:{
          schema:1,slot,mode:config.mode,
          provider:gpuSource?(onceSize?'NATIVE_WGC_ON_DEMAND':'NATIVE_WGC_PERSISTENT'):'CUA_DRIVER_WINDOW_CAPTURE',
          observedWindow:{pid:after.pid,windowId:after.windowId},
          processIdentity:{pid:after.pid,windowId:after.windowId,
            createdFileTime:after.createdFileTime,sessionId:after.sessionId},
          width:frame.width,height:frame.height,
          sha256:frame.sha256??null,
          capturedAtMs:gpuSource?frame.capturedAtMs:null,
          captureStartedAtMs:gpuSource?null:captureStartedAtMs,
          workerSequence:gpuSource?frame.workerSequence:null,
          sourceTimeKind:gpuSource?'WGC_READBACK_COMPLETED':
            'CUA_QUERY_STARTED',
          frameFreshness:gpuSource ?
            'WGC_RECENT_TIMESTAMP_NOT_GAMEPLAY_PROOF' :
            'UNVERIFIED_SINGLE_FRAME',
          background:!before.foreground && !after.foreground,
          screenshotStatus:webp?'WEBP_RETURNED_NOT_GAMEPLAY_PROOF':'PNG_RETURNED_NOT_GAMEPLAY_PROOF',
          ...(webp?{mimeType:'image/webp',preview:onceSize?'ON_DEMAND':'SAMPLER'}:{}),
          gameInput:'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED',
          note:'Pixels may be stale, blank, or unrepresentative of gameplay. Captures do not prove player login or peer-to-peer join.',
        },
        imageBase64:null,
      };
      if(onceSize) {
        // Compare the fresh, exact-target encoded hash BEFORE Base64. Raw
        // bytes are not returned to MCP or retained in a second cache.
        const meta=observation.metadata,identity=meta.processIdentity;
        if(!/^[a-f0-9]{64}$/.test(meta.sha256??''))throw new Error('GPU_PREVIEW_UNAVAILABLE');
        const frameRef=createHash('sha256').update(JSON.stringify([
          slot,identity.pid,identity.windowId,identity.createdFileTime,identity.sessionId,
          meta.mimeType,meta.width,meta.height,meta.sha256,
        ])).digest('hex');
        const unchanged=afterFrame===frameRef;
        return {metadata:{...meta,frameRef,hasNewImage:!unchanged,
          ...(unchanged?{screenshotStatus:'WEBP_UNCHANGED_NOT_GAMEPLAY_PROOF'}:{})},
          imageBase64:unchanged?null:Buffer.isBuffer(frame.bytes)?
            frame.bytes.toString('base64'):frame.imageBase64};
      }
      if(deferred)observation.imageRef=frame.imageRef;
      else observation.imageBase64=frame.imageBase64;
      // No additional native read, frame parse or player/world inference.
      // This is only a bounded capability hint on an existing screenshot.
      if(webp)return observation; // No unrelated legacy F3 projection in AI previews.
      // The observation is newly created here and has not been shared.
      observation.metadata.f3VisualReview=projectF3VisualReview(observation.metadata);
      return observation;
    } finally {
      visualReadBusy=false;
    }
  }
  const visualObserver=visualObserverFactory({
    capture:slot=>captureVerifiedFrame(slot,null,undefined,true),
    sourceLabel:gpuSource?'PERSISTENT_WGC_GPU_FRAME_SOURCE':'BOUNDED_CUA_WINDOW_SNAPSHOTS',
    onFinish:(slot)=>{
      gpuSource?.stop([slot]);
      revokeObservedRound(slot);
    },
    onCaptureFailure:(slot)=>revokeObservedRound(slot),
  });
  function checkedSlots(slots) {
    if (!Array.isArray(slots) || slots.length<1 || slots.length>5 ||
        new Set(slots).size!==slots.length) {
      throw new Error('Choose 1-5 distinct Minecraft profiles');
    }
    for (const slot of slots) protectedSlot(slotOf(slot));
    return slots;
  }

  // The actual visual tool handlers, not a second public descriptor catalog.
  const tools={
    async start_visual_observation(args) {
      keysExactly(args,['slots','intervalMs','durationMs','imageFormat','width','height']);
      const slots=checkedSlots(args.slots);
      const webpSize=visualSize(args);
      if (!gpuSource && !config.cuaExecutable) {
        throw new Error('Visual provider is NOT_CONFIGURED');
      }
      const durationMs=args.durationMs??OBSERVATION_LIMITS.defaultDurationMs;
      const intervalMs=args.intervalMs??
        (gpuSource?500:OBSERVATION_LIMITS.defaultIntervalMs);
      if (!gpuSource) {
        return visualObserver.start(slots,{intervalMs,durationMs});
      }
      // Refuse invalid cadence before any native inspection or GPU spawn;
      // the observer's own bounds still validate all provider modes.
      if (!Number.isInteger(intervalMs) ||
          intervalMs<OBSERVATION_LIMITS.minimumIntervalMs ||
          intervalMs>OBSERVATION_LIMITS.maximumIntervalMs ||
          !Number.isInteger(durationMs) ||
          durationMs<OBSERVATION_LIMITS.minimumDurationMs ||
          durationMs>OBSERVATION_LIMITS.maximumDurationMs) {
        throw new TypeError('Visual observation timing is outside bounded range');
      }
      // The WGC producer needs spare frames, not a fixed 5 FPS for a slow
      // on-demand reader. Keep up to 2x observer cadence, never above old 5.
      const fps=Math.min(5,Math.max(1,Math.ceil(2000/intervalMs)));
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      setNativeBusy(true);
      const startup={slots:[...slots],cancelled:false};
      gpuStartup=startup;
      const started=[];
      try {
        // Verify every owner before starting the optional read-only workers.
        // If one fails, stop every worker started in this request. No game
        // processes, worlds or authenticated controller instances are created.
        for (const slot of slots) {
          if (startup.cancelled) throw new Error('GPU_OBSERVATION_START_CANCELLED');
          const target=await readNative('target',slot);
          if (startup.cancelled) throw new Error('GPU_OBSERVATION_START_CANCELLED');
          await gpuSource.start(slot,target,{durationMs,fps,...(webpSize?{webpSize}:{})});
          started.push(slot);
          if (startup.cancelled) throw new Error('GPU_OBSERVATION_START_CANCELLED');
        }
        const response=visualObserver.start(slots,{intervalMs,durationMs});
        return {...response,source:'PERSISTENT_NATIVE_WGC_EXPLICIT_OPT_IN',
          ...(webpSize?{mimeType:'image/webp',...webpSize}:{}),
          pixelsPassedToModel:'ONLY_ON_MCP_READ'};
      } catch(error) {
        gpuSource.stop(started);
        throw error;
      } finally {
        if (gpuStartup===startup) gpuStartup=null;
        setNativeBusy(false);
      }
    },
    async read_visual_observation(args) {
      keysExactly(args,['slot','afterSequence','includeImage','screenReview','imageFormat']);
      if(args.imageFormat!==undefined&&!['png','webp'].includes(args.imageFormat))throw new Error('Invalid tool arguments');
      const slot=slotOf(args.slot);
      protectedSlot(slot);
      // An AI may submit one visually transcribed Position line only AFTER
      // an earlier explicit image read. The review itself returns text only.
      if (args.screenReview!==undefined && args.includeImage!==false) {
        throw new Error('Screen text review requires includeImage=false');
      }
      if (args.includeImage!==undefined && typeof args.includeImage!=='boolean') {
        throw new Error('Invalid tool arguments');
      }
      const afterSequence=args.afterSequence??0;
      const wantsImage=args.screenReview===undefined && args.includeImage!==false;
      // First inspect metadata ONLY. Never record a PNG as delivered before
      // checking that the original native child still owns its PID/HWND.
      const first=visualObserver.read(slot,{afterSequence,includeImage:false});
      let observation=first;
      if(first.metadata.freshFrameAvailable===true&&args.imageFormat!==undefined&&
         (first.metadata.mimeType??'image/png')!=='image/'+args.imageFormat)throw new Error('VISUAL_FORMAT_MISMATCH');
      const mustRevalidate=first.metadata.freshFrameAvailable===true &&
        (args.screenReview!==undefined ||
         (wantsImage && first.metadata.sequence>afterSequence));
      if(mustRevalidate){
        let live;
        try {
          live=await readNative('target',slot);
        } catch {
          // No native identity proof means no cached PNG or screen-text
          // review, even if the timestamp still falls within 3.5 seconds.
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        requireOpen();
        if(!sameNativeVisualTarget(live,first.metadata.captureIdentity)){
          // Positive replacement: revoke just this slot's stale observer
          // frame/worker. Native game process and other slots are untouched.
          visualObserver.stop([slot]);
          gpuSource?.stop([slot]);
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        // A sampler may advance/revoke its cache during the native await.
        // Check continuity using metadata ONLY: a rejected new frame must
        // never be recorded as "already delivered" before the AI sees it.
        const verified=visualObserver.read(slot,{afterSequence,includeImage:false});
        const a=first.metadata,b=verified.metadata;
        if(b.freshFrameAvailable!==true||
           b.sequence!==a.sequence||
           b.successfulSamples!==a.successfulSamples||
           b.collectedAt!==a.collectedAt||
           b.sourceFrameAtMs!==a.sourceFrameAtMs||
           b.sourceTimeKind!==a.sourceTimeKind||
           b.producerSequence!==a.producerSequence||b.mimeType!==a.mimeType||
           !sameNativeVisualTarget(b.captureIdentity,a.captureIdentity)){
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        // Both reads above were metadata-only. With no async yield between
        // this check and the actual PNG read, only the same validated cache
        // frame may become deliverable and eligible for a later text review.
        observation=wantsImage?
          visualObserver.read(slot,{afterSequence,includeImage:true}):verified;
        if(wantsImage && (observation.metadata.hasNewImage!==true||
           observation.metadata.sequence!==b.sequence||
           !sameNativeVisualTarget(
             observation.metadata.captureIdentity,b.captureIdentity)||
           typeof observation.imageBase64!=='string')){
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
      }
      // No new image or usable frame: return the first metadata-only
      // receipt. Never attempt an unfenced second read.
      // This remains an ephemeral cache READ, not a screenshot, game input,
      // player/world attribution or automatic clipboard operation.
      const metadata={...observation.metadata,
        ...(observation.metadata.mimeType==='image/webp'?{}:
          {f3VisualReview:projectF3VisualReview(observation.metadata)})};
      if(args.screenReview!==undefined){
        metadata.f3ScreenXYZ=reviewVisibleXYZFromFrame(metadata,args.screenReview);
      }
      return {...observation,metadata};
    },
    async stop_visual_observation(args) {
      keysExactly(args,['slots']);
      const slots=checkedSlots(args.slots);
      const cancelStartup=gpuStartup!==null &&
        slots.some(slot=>gpuStartup.slots.includes(slot));
      if (cancelStartup) {
        // Cancel the entire unfinished batch. Slots not yet started will
        // fail admission after the current await instead of spawning later.
        gpuStartup.cancelled=true;
        gpuSource?.stop(gpuStartup.slots);
      }
      const response=visualObserver.stop(slots);
      gpuSource?.stop(slots);
      return cancelStartup ? {...response,
        status:'STARTUP_CANCELLATION_REQUESTED',
        startOutcome:'UNCONFIRMED_UNTIL_START_REQUEST_SETTLES'} : response;
    },
    async assess_client_background_action(args) {
      keysExactly(args,['slot','elementIndex','action']);
      const slot=slotOf(args.slot);
      protectedSlot(slot);
      // Reject malformed intent before any native/UIA observation; clients
      // cannot upgrade an assessment with a token, fallback or retry flag.
      // This one shared validator is also used by the pure assessor.
      const intent=validateCuaStrictBackgroundIntent({
        elementIndex:args.elementIndex,action:args.action,
      });
      // Always refresh via the existing identity-fenced read. A previous
      // tool result or caller-supplied UIA label is never a current action
      // authority. No input API is called from this read-only assessor.
      const inspected=await callTool('inspect_client_ui',{slot});
      return assessCuaStrictBackgroundAction(inspected,intent);
    },
    async inspect_client_ui(args) {
      keysExactly(args,['slot']);
      const slot=slotOf(args.slot);
      protectedSlot(slot);
      if(!config.cuaExecutable) {
        return {schema:1,slot,status:'BACKGROUND_UNAVAILABLE',
          reason:'CUA_UIA_NOT_CONFIGURED',
          screenshotCaptured:false,actionPermitted:false,
          gameplayInput:'NOT_VERIFIED'};
      }
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      // Use the existing one-native-read gate. This semantic read neither
      // owns game lifecycle nor introduces a second input/visual scheduler.
      visualReadBusy=true;
      try {
        const before=await readNative('target',slot);
        requireOpen();
        const ui=await inspectCua(config.cuaExecutable,before);
        requireOpen();
        const after=await readNative('target',slot);
        requireOpen();
        if(!sameNativeVisualTarget(after,before)) {
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        return {...ui,slot,identity:{pid:after.pid,windowId:after.windowId,
          createdFileTime:after.createdFileTime,sessionId:after.sessionId}};
      } finally {visualReadBusy=false;}
    },
    async observe_client(args) {
      keysExactly(args,['slot','imageFormat','width','height','afterFrame']);
      const slot=slotOf(args.slot);
      protectedSlot(slot);
      if(args.afterFrame!==undefined&&(args.imageFormat!=='webp'||
         typeof args.afterFrame!=='string'||!/^[a-f0-9]{64}$/.test(args.afterFrame))) {
        throw new Error('Invalid tool arguments');
      }
      const size=visualSize(args);
      const result=await captureVerifiedFrame(slot,size,args.afterFrame);
      if(args.imageFormat==='png'&&result.metadata.mimeType==='image/webp')throw new Error('VISUAL_FORMAT_MISMATCH');
      return result;
    }
  };
  return {
    tools,checkedSlots,
    isBusy:()=>visualReadBusy,
    readMetadata:slots=>slots.map(slot=>visualObserver.read(slot,{includeImage:false}).metadata),
    revokeSlot:slot=>{
      visualObserver.stop([slot]);
      gpuSource?.stop([slot]);
    },
    activity:()=>typeof visualObserver.activity==='function'?visualObserver.activity():[],
    source:()=>gpuSource?'WGC':config.cuaExecutable?'CUA':'NOT_CONFIGURED',
    shutdown:()=>{
      if(gpuStartup)gpuStartup.cancelled=true;
      visualObserver.shutdown();
      gpuSource?.shutdown();
    },
  };
}
