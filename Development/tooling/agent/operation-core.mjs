#!/usr/bin/env node
// Shared local agent operation owner. No transport, arbitrary code runner or independent game controller.
// Every native mutation goes through the existing authenticated Lazy Tester controller.
import {createHash,randomBytes} from 'node:crypto';
import path from 'node:path';
import {planPeerSession,planPlayerActions,expandActionGroups} from './control-plan.mjs';
import {compilePlayerInputTimeline,compilePerClientInputFrames} from './input-timeline.mjs';
import {assessInputRoute,requiredInputSlots} from './input-route.mjs';
import {inputExecutionStatus} from './input-delivery.mjs';
import {projectAgentActivity} from './activity-status.mjs';
import {prepareMc02PulseBatch,prepareMultiClientPulseBatches} from './mc02-action-sequence.mjs';
import {summarizeGameEvidence,summarizeEvidenceReports,normalizeEvidenceSlots} from './evidence-summary.mjs';
import {captureCuaWindow,inspectCuaWindowElements,assessCuaStrictBackgroundAction,validateCuaStrictBackgroundIntent,validateCuaExecutable} from './visual-capture.mjs';
import {validateDiagnosticsRoot,diagnosticsOverview,listDiagnosticIssues,
  getDiagnosticIssue,queryDiagnosticEvidence} from './diagnostics-local.mjs';
import {validateTestingArchiveRoot,archiveTestingResult,
  readTestingArchive,archivedTestingSlotReference,reviewTestingArchiveForRun}
  from './testing-result-store.mjs';
import {createVisualObserver,OBSERVATION_LIMITS,
  summarizeTeamVisualState,verifyTeamObservationBasis,
  evaluateTeamActionFeedback,summarizeTeamCoordination,
  projectF3VisualReview,reviewVisibleXYZFromFrame} from './visual-observer.mjs';
import {createGpuFrameSource,validateGpuExecutable} from './gpu-frame-source.mjs';
import {readPinnedProbeCatalogCached,probeSourceRevisionStatus,
  reviewProbeRun,compactProbeQaResult,
  probeQaProofGate} from './probe-qa.mjs';
import {readMapTestContractCached,mapContractSourceStatus,planMapTests,
  compactMapTestPlan,composeMapQaReport} from './map-test-plan.mjs';

import {slotOf,keysExactly,assertAgentSurface} from './agent-contract.mjs';
import {validateNativeWindowTarget,
  sameNativeVisualTarget,
  validateNativeReport,
  testingRecoveryNextAction,
  inspectTestingRunNative,
  coordinateTestingRunNative,
  publishAgentActivityNative,
  runNative,
  sendExperimentalMc02Native,
  sendExperimentalAiNative,
  sendExperimentalMc02BatchNative,
  canReuseVerifiedMc02BatchNeutral,
  confirmedObsoleteMc02BatchCleanup,
  makeAiLeaseEpoch,
  sendExperimentalAiWriterLeaseNative,
  canReuseVerifiedNativeBatchNeutral,
  sendExperimentalMultiClientBatchNative,
  computerUseTarget,
  mc02ButtonMask,
  definitelyDeniedWriterLease} from './native-transport.mjs';
import {readGameEvidence,EVIDENCE_LIMIT,validCaptureCursor} from './evidence-reader.mjs';
import {toolsForMode} from './tool-catalog.mjs';

// Preserve the previous public source API through mcp.mjs.
export {REQUEST_LIMIT,boundedMcpText,publicToolError} from './agent-contract.mjs';
export {validateNativeWindowTarget,
  sameNativeVisualTarget,
  validateNativeReport,
  parseTestingInspectionReceipt,
  testingRecoveryNextAction,
  inspectTestingRunNative,
  coordinateTestingRunNative,
  publishAgentActivityNative,
  runNative,
  sendExperimentalMc02Native,
  sendExperimentalAiNative,
  sendExperimentalMc02BatchNative,
  canReuseVerifiedMc02BatchNeutral,
  confirmedObsoleteMc02BatchCleanup,
  makeAiLeaseEpoch,
  sendExperimentalAiWriterLeaseNative,
  matchesNativeMultiClientBatchReceipt,
  canReuseVerifiedNativeBatchNeutral,
  sendExperimentalMultiClientBatchNative} from './native-transport.mjs';
export {parseCapturedLine,readGameEvidence} from './evidence-reader.mjs';
export {TOOL_LIST} from './tool-catalog.mjs';

export const MAX_MCP_IMAGE_CHARS = OBSERVATION_LIMITS.maximumEncodedFrameChars; // Canonical 3 MiB lossless PNG -> 4 MiB base64

export function parseConfiguration(argv, env = process.env) {
  let mode = 'human-plus-four-ai';
  let surface = 'legacy';
  let executable = env.LAZY_TESTER_AGENT_EXE ?? '';
  let evidenceDirectory = env.LAZY_TESTER_EVIDENCE_DIR ?? '';
  let diagnosticsDirectory = env.LAZY_TESTER_DIAGNOSTICS_DIR ?? '';
  let testingArchiveDirectory = env.LAZY_TESTER_TESTING_ARCHIVE_DIR ?? '';
  let probeZip = env.LAZY_TESTER_PROBE_ZIP ?? '';
  let mapContractPath = env.LAZY_TESTER_MAP_CONTRACT ?? '';
  let cuaExecutable = env.LAZY_TESTER_CUA_EXE ?? '';
  let gpuExecutable = env.LAZY_TESTER_GPU_CAPTURE_EXE ?? '';
  let experimentalMc02InputEnabled=false;
  let experimentalMultiClientInputEnabled=false;
  for (let i=0;i<argv.length;i++) {
    const flag=argv[i];
    if (!['--surface','--mode','--exe','--evidence-dir','--diagnostics-dir','--testing-archive-dir','--probe-zip','--map-contract','--cua-exe','--gpu-capture-exe','--mc02-input-experimental','--multi-client-input-experimental'].includes(flag) || i===argv.length-1) {
      throw new Error('Accepted arguments: [--surface legacy|code-mode|playtest] --mode human-plus-four-ai|five-ai --exe <lazy-tester.exe> [--evidence-dir <folder>] [--diagnostics-dir <project|Data/Diagnostics>] [--testing-archive-dir <project|Data/Testing>] [--map-contract <map-test-contract.json>] [--cua-exe <cua-driver.exe>] [--gpu-capture-exe <lazy-tester-gpu-capture.exe>]');
    }
    const value=argv[++i];
    if (flag==='--mode') mode=value;
    if (flag==='--surface') surface=value;
    if (flag==='--exe') executable=value;
    if (flag==='--evidence-dir') evidenceDirectory=value;
    if (flag==='--diagnostics-dir') diagnosticsDirectory=value;
    if (flag==='--testing-archive-dir') testingArchiveDirectory=value;
    if (flag==='--probe-zip') probeZip=value;
    if (flag==='--map-contract') mapContractPath=value;
    if (flag==='--cua-exe') cuaExecutable=value;
    if (flag==='--gpu-capture-exe') gpuExecutable=value;
    if (flag==='--mc02-input-experimental') {
      if(value!=='on') throw new Error('MC02 experimental input requires explicit on');
      experimentalMc02InputEnabled=true;
    }
    if (flag==='--multi-client-input-experimental') {
      if(value!=='on')throw new Error('Multi-client input opt-in requires explicit on');
      experimentalMultiClientInputEnabled=true;
    }
  }
  assertAgentSurface(surface,experimentalMc02InputEnabled||experimentalMultiClientInputEnabled);
  if(experimentalMc02InputEnabled&&experimentalMultiClientInputEnabled) {
    throw new Error('Select only one experimental native input mode');
  }
  if (!['human-plus-four-ai','five-ai'].includes(mode)) throw new Error('Unknown test mode');
  if(experimentalMc02InputEnabled &&
     (mode!=='human-plus-four-ai' ||
      env.LAZY_TESTER_MC02_VIRTUAL_INPUT!=='I_ACCEPT_EXPERIMENTAL_MC02_INPUT')) {
    throw new Error('MC02 experiment needs human-plus-four-ai mode and explicit native opt-in');
  }
  if(experimentalMultiClientInputEnabled &&
     env.LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT!=='I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT') {
    throw new Error('Multi-client input requires explicit native opt-in');
  }
  if (executable && (!path.isAbsolute(executable) || !/\.exe$/i.test(executable))) {
    throw new Error('Set --exe to the absolute path of the installed lazy-tester.exe');
  }
  if (evidenceDirectory && !path.isAbsolute(evidenceDirectory)) {
    throw new Error('Evidence directory must be absolute');
  }
  if (diagnosticsDirectory || testingArchiveDirectory) {
    // Match the native App/../Data layout only. Do not guess a root for a
    // developer build, legacy flat install, or archive-only invocation.
    const appFolder=executable ? path.dirname(executable) : '';
    const projectData=appFolder && path.basename(appFolder).toLowerCase()==='app'
      && path.basename(executable).toLowerCase()==='lazy-tester.exe'
      && path.normalize(executable)===executable
      ? path.join(path.dirname(appFolder),'Data') : '';
    const archivePath=(value,child)=>{
      if(value!=='project') return value;
      if(!projectData)throw new Error('Project archive selection requires an absolute App/lazy-tester.exe path');
      return path.join(projectData,child);
    };
    diagnosticsDirectory=archivePath(diagnosticsDirectory,'Diagnostics');
    testingArchiveDirectory=archivePath(testingArchiveDirectory,'Testing');
    if (diagnosticsDirectory) validateDiagnosticsRoot(diagnosticsDirectory);
    if (testingArchiveDirectory)validateTestingArchiveRoot(testingArchiveDirectory);
    // Configuration consistency, not filesystem identity or access authority.
    // Existing archive readers/writers still own their per-operation checks.
    const roots=[diagnosticsDirectory,testingArchiveDirectory].filter(Boolean).map(dir=>path.dirname(dir));
    const expected=projectData || roots[0];
    if(roots.some(root=>path.relative(expected,root)!=='')) {
      throw new Error('Diagnostics and Testing must use the same Project Data folder');
    }
  }
  if (probeZip && (!path.isAbsolute(probeZip) || !/\.zip$/i.test(probeZip))) {
    throw new Error('Probe ZIP must be an absolute .zip file path');
  }
  if (mapContractPath && (!path.isAbsolute(mapContractPath) ||
      path.extname(mapContractPath).toLowerCase()!=='.json')) {
    throw new Error('Map contract must be an absolute .json file path');
  }
  if (cuaExecutable) validateCuaExecutable(cuaExecutable);
  if (gpuExecutable) validateGpuExecutable(gpuExecutable);
  return Object.freeze({mode, surface, executable, evidenceDirectory,diagnosticsDirectory,
    testingArchiveDirectory,probeZip,mapContractPath,
    cuaExecutable,gpuExecutable,experimentalMc02InputEnabled,
    experimentalMultiClientInputEnabled});
}

// SHA-256 of the deterministic canonical pulse plan. Prevents accidental
// evaluation against a different action list; it does not authenticate an
// externally copied or fabricated receipt and proves no Minecraft effect.
export function fingerprintActions(mode,actions) {
  return fingerprintPulsePlan(mode,prepareMultiClientPulseBatches(mode,actions));
}
function fingerprintPulsePlan(mode,plan) {
  const projected=plan.batches.map(batch=>({
    slot:batch.slot,pulses:batch.pulses.map(pulse=>({
      atMs:pulse.atMs,keys:pulse.keys,dx:pulse.dx,dy:pulse.dy,ttlMs:pulse.ttlMs,
    })),
  }));
  return createHash('sha256').update(JSON.stringify({mode,projected}))
    .digest('hex');
}

function capabilities(mode,experimental=false,multi=false) {return {schema:1,mode,hostSlot:1,humanSlots:mode==='human-plus-four-ai'?[1]:[],
  aiSlots:mode==='human-plus-four-ai'?[2,3,4,5]:[1,2,3,4,5],
  available:toolsForMode(experimental,multi).filter(tool=>tool.name!=='agent_capabilities').map(tool=>tool.name),
  notAvailable:['parallel_independent_keyboard_mouse','automatic_world_host_join','send_minecraft_command'],
  visualCapture:'OPTIONAL_CUA_OR_PERSISTENT_WGC_SOURCE_NOT_RUNTIME_VERIFIED',
  independentInput:{state:'BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND',
    simultaneousIntentChannels:true,actualInputEnabled:false,
    experimentalMc02Win32PollingOnly:experimental?'OPTED_IN_NOT_GAMEPLAY_VERIFIED':'DISABLED',
    experimentalMultiClientNative:multi?'OPTED_IN_SOURCE_CANDIDATE_NOT_GAMEPLAY_VERIFIED':'DISABLED',
    requiresHumanFocusProtection:mode==='human-plus-four-ai',
    nativeWriterLease:multi?'CONTROLLER_OWNED_EPHEMERAL_SLOT_LEASE':
      'DISABLED',
    startupReconciliation:multi?
      'EXPLICIT_NEUTRAL_OR_AUTHENTICATED_NEW_AI_LAUNCH':
      'EXPERIMENT_DISABLED',
    requiresPerClientEmergencyRelease:true,
    nativeSessionModel:'SAME_WINDOWS_SESSION_ONLY',
    windowsAgentWorkspace:'PREVIEW_NOT_INTEGRATED_NO_CROSS_SESSION_INPUT',
    transactionEngine:inputExecutionStatus()},
  note:'MCP tools control the native session owner and read evidence; they do NOT independently move multiple background Minecraft clients.'};}

/**
 * One conservative next-action hint for the original probe_qa response.
 * AI should use this rather than inventing a second Testing workflow or
 * treating a source-only CHECKPOINT as a game action.
 * This is not a second result/acceptance classification or dispatch gate.
 */
export function operatorNextTestingAction(result) {
  if(!result||typeof result!=='object')return 'REVIEW_UNAVAILABLE_TESTING_STATUS';
  if(result.terminal===true) {
    if(result.status==='PROBE_REPORTED_PASS')
      return 'REVIEW_REPORTED_PROBE_PASS_MAP_NOT_TESTED';
    if(result.status==='FAIL')return 'REVIEW_REPORTED_PROBE_FAILURES';
    if(result.status==='NEEDS_REVIEW')return 'REVIEW_REPORTED_PROBE_NOTES';
    return 'REVIEW_BLOCKED_OR_INCOMPLETE_TERMINAL_REPORT';
  }
  if(result.phase==='AWAITING_AUTHORIZED_COMMAND')
    return 'AWAIT_AUTHORIZED_PLAYER_COMMAND_AND_QA_WORLD';
  if(result.phase==='PREFLIGHT')
    return 'RESOLVE_APPROVED_PROBE_SOURCE_OR_CAPTURE';
  if(result.status==='WAITING')
    return 'WAIT_FOR_NEW_EVIDENCE_THEN_CALL_SAME_TOOL';
  if(result.reason==='QA_OPERATION_IN_PROGRESS')
    return 'WAIT_FOR_CURRENT_TESTING_READ';
  if(result.status==='INCOMPLETE')
    return 'REVIEW_INCOMPLETE_EVIDENCE';
  if(result.status==='BLOCKED')
    return 'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART';
  return 'REVIEW_UNAVAILABLE_TESTING_STATUS';
}

export function createOperationCore(config,{
  native=runNative,evidence=readGameEvidence,capture=captureCuaWindow,
  inspectCua=inspectCuaWindowElements,
  mc02Input=sendExperimentalMc02Native,
  mc02Batch=sendExperimentalMc02BatchNative,
  aiInput=sendExperimentalAiNative,
  aiMultiBatch=sendExperimentalMultiClientBatchNative,
  aiLease=sendExperimentalAiWriterLeaseNative,
  gpuSourceFactory=createGpuFrameSource,
  visualObserverFactory=createVisualObserver,
  publishActivity=publishAgentActivityNative,
  coordinateTesting=coordinateTestingRunNative,
  inspectTesting=inspectTestingRunNative,
  archiveTesting=archiveTestingResult,
  readArchivedTesting=readTestingArchive,
  findArchivedTestingSlot=archivedTestingSlotReference,
  reviewTestingArchive=reviewTestingArchiveForRun,
  probeCatalog=readPinnedProbeCatalogCached,
  probeRevisionStatus=probeSourceRevisionStatus,
  mapContract=readMapTestContractCached,
  mapRevisionStatus=mapContractSourceStatus,
  qaClock=()=>Date.now(),
}={}) {
  // The canonical transport already validates/projects its fresh response.
  // Injected providers must pass the same validators before any consumer.
  // Select once; never use a caller-supplied flag to bypass validation.
  const readNative=native===runNative
    ? (operation,slot)=>runNative(config.executable,operation,slot)
    : async (operation,slot)=>{
        const receipt=await native(config.executable,operation,slot);
        return operation==='target'?
          validateNativeWindowTarget(receipt,slot):validateNativeReport(receipt,operation);
      };
  let inFlight=false; // mutating or exclusive native operation
  // Stdio closure fences pending preflight/lease continuations in THIS
  // existing MCP handler. It does not own game lifecycle or recovery.
  let closed=false;
  const requireOpen=()=>{
    if(closed)throw new Error('MCP_CONNECTION_CLOSED');
  };
  // One volatile plan per slot; not a Minecraft world manager or an auto-run loop.
  const probeQaPlans=new Map();
  // At most one local evidence baseline per slot, separate from Probe QA.
  // No persistent run store, player attribution, or background collector.
  const evidenceBaselines=new Map();
  let evidenceWindowBusy=false;
  // Only correlation identifiers, never a second checkpoint or outcome
  // database. Native retained actor decides whether a slot is owned.
  const testingRunIds=new Map();
  const testingOwnerId=randomBytes(16).toString('hex');
  const probeQaBusy=new Set(); // short per-slot tool admission, no background runner
  // Source-only terminal snapshots live in the same per-slot plan. No disk
  // database, background processing, or extra QA session authority.
  function settleProbeQa(slot,plan,report) {
    if(probeQaPlans.get(slot)!==plan) {
      return {schema:1,slot,status:'BLOCKED',
        reason:'QA_PLAN_SUPERSEDED',gameplayAcceptance:'NOT_VERIFIED'};
    }
    if(plan.terminalReport)return plan.terminalReport;
    // Pure final projection: existing Probe parser owns all foundation
    // outcomes; map requirements remain NOT_TESTED. No extra ZIP/native/
    // capture read, second QA engine, game command or output archive.
    const qaReport=plan.mapPlan?
      composeMapQaReport(plan.mapPlan,report,plan.proofGate):null;
    const frozen=Object.freeze({...report,
      ...(plan.proofGate?{proofGate:plan.proofGate}:{}),
      ...(plan.mapPlan && !Object.hasOwn(report,'mapPlan')?{mapPlan:plan.mapPlan}:{}),
      ...(qaReport?{qaReport}:{}),
      terminal:true,
      ...(report.counts?{counts:Object.freeze({...report.counts})}:{}),
      ...(report.missingIds?{missingIds:Object.freeze([...report.missingIds])}:{})});
    plan.terminalReport=frozen;
    return frozen;
  }
  let visualReadBusy=false; // only one identity-pinned screenshot at a time
  let mc02SequenceAbort=null; // pending opted-in legacy MC02 request only
  let multiSequence=null; // one bounded cross-slot request, not a game owner
  // Volatile MCP-process fail-closed input markers, not another game
  // controller, OS channel owner, retry queue or account state registry.
  const uncertainAiSlots=new Set();
  // A brand-new opted-in MCP process cannot know whether a previous MCP
  // process died with active/uncertain input. Hold all AI-owned slots until
  // an explicit exact-owner neutral ACK OR a verified new AI game launch.
  // No native requests or game writes occur merely by starting MCP.
  const startupReconcileSlots=new Set();
  // One transient observed-action round; no second native writer or observer.
  // Carry the exact delivered PNG basis across the asynchronous writer lease.
  let observedSequence=null;
  const markAiUncertain=slots=>{
    for(const slot of slots) {
      uncertainAiSlots.add(slot);
      // Once an input sequence may have been sent, its failed transport
      // requires neutral recovery even if this was a reconnecting slot.
      startupReconcileSlots.delete(slot);
    }
  };
  const refuseUncertain=slots=>{
    if(slots.some(slot=>uncertainAiSlots.has(slot))) {
      throw new Error('AI_INPUT_RECOVERY_REQUIRED');
    }
  };
  const exclusiveBusy=()=>inFlight||visualReadBusy;
  let gpuStartup=null; // bounded request only; not game/session lifecycle
  const mc02Enabled=config.experimentalMc02InputEnabled===true &&
    config.mode==='human-plus-four-ai';
  const multiEnabled=config.experimentalMultiClientInputEnabled===true;
  // Random stable ID for this ONE MCP stdio process; passed to native
  // CLI children only via private process environment, never argv.
  const writerId=multiEnabled?randomBytes(16).toString('hex'):null;
  if(multiEnabled) {
    const aiSlots=config.mode==='five-ai'?[1,2,3,4,5]:[2,3,4,5];
    for(const slot of aiSlots) {
      startupReconcileSlots.add(slot);
      uncertainAiSlots.add(slot);
    }
  }
  const gpuSource=config.gpuExecutable ?
    gpuSourceFactory({executable:config.gpuExecutable}):null;
  function protectedSlot(slot) {
    if (config.mode==='human-plus-four-ai' && slot===1) {
      throw new Error('MC01 human host is protected from AI visual observation');
    }
  }
  async function captureVerifiedFrame(slot) {
    protectedSlot(slot);
    if (!config.cuaExecutable && !gpuSource) {
      throw new Error('Visual provider is NOT_CONFIGURED; set --cua-exe or --gpu-capture-exe');
    }
    // A verified read-only capture may accompany an already-running
    // experimental MC02 sequence. It must never clear that sequence's
    // native mutation ownership on completion.
    const alongsideSequence=mc02SequenceAbort!==null||multiSequence!==null;
    if(visualReadBusy || (inFlight && !alongsideSequence)) {
      throw new Error('Native operation in progress; do not overlap requests');
    }
    // This read-only gate is sufficient to block new native mutations
    // via exclusiveBusy(). Never claim ownership of inFlight: a separate
    // urgent neutral-state write may overlap a screenshot and must keep
    // its own mutation lock until its authenticated request completes.
    visualReadBusy=true;
    try {
      const before=await readNative('target',slot);
      // GPU provider is a persistent producer started by the same explicit
      // visual-observation MCP action. No fallback to desktop/global capture.
      // Read ONLY from the WGC worker pinned to this same exact native
      // process/window/birth/session. Without this check a restarted slot
      // could receive its predecessor's cached frame.
      // A Cua screenshot has no trustworthy pixel-presentation timestamp.
      // The query-start time is a conservative lower bound on WHEN a
      // screenshot request began (not when the game rendered its pixels).
      // WGC carries a separate producer-authored readback time instead.
      const captureStartedAtMs=Date.now();
      const frame=gpuSource ? gpuSource.latest(slot,before) :
        await capture(config.cuaExecutable,before);
      const after=await readNative('target',slot);
      if (before.pid!==after.pid || before.windowId!==after.windowId ||
          before.createdFileTime!==after.createdFileTime ||
          before.sessionId!==after.sessionId) {
        throw new Error('Minecraft window changed during capture; screenshot discarded');
      }
      if (!frame || frame.mimeType!=='image/png' ||
          typeof frame.imageBase64!=='string' ||
          !Number.isInteger(frame.width) || !Number.isInteger(frame.height) ||
          frame.width<1 || frame.height<1) {
        throw new Error('Cua did not produce a usable PNG frame');
      }
      const observation = {
        metadata:{
          schema:1,slot,mode:config.mode,
          provider:gpuSource?'NATIVE_WGC_PERSISTENT':'CUA_DRIVER_WINDOW_CAPTURE',
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
          screenshotStatus:'PNG_RETURNED_NOT_GAMEPLAY_PROOF',
          gameInput:'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED',
          note:'Pixels may be stale, blank, or unrepresentative of gameplay. Captures do not prove player login or peer-to-peer join.',
        },
        imageBase64:frame.imageBase64,
      };
      // No additional native read, frame parse or player/world inference.
      // This is only a bounded capability hint on an existing screenshot.
      return {
        ...observation,
        metadata:{...observation.metadata,
          f3VisualReview:projectF3VisualReview(observation.metadata)},
      };
    } finally {
      visualReadBusy=false;
    }
  }
  const revokeObservedRound=slot=>{
    if(observedSequence?.slots.includes(slot)) {
      // A single missing/revoked captured frame source invalidates
      // the entire SELECTED action round. Never silently proceed with
      // the remaining AI clients or start a retry.
      markAiUncertain(observedSequence.slots);
      multiSequence?.abort.abort();
    }
  };
  const visualObserver=visualObserverFactory({
    capture:captureVerifiedFrame,
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
  async function changeAiLease(operation,slots,leaseEpoch,
                               neutralOnly=false) {
    if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
    const result=await aiLease(config.executable,operation,slots,{
      writerId,mode:config.mode,leaseEpoch,neutralOnly,
    });
    if(result?.schema!==1||
       JSON.stringify(result?.slots)!==JSON.stringify(slots)||
       result?.status!==(operation==='acquire'?
         'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED')||
       result?.gameplayObserved!==false) {
      throw new Error('AI_WRITER_LEASE_UNCONFIRMED');
    }
  }
  async function releaseAiSlots(slots,leaseEpoch) {
    // SERIAL independent original-channel neutral writes. A dead AI
    // window must not hide ACKs from other still-live game slots.
    // Neither bucket is proof that Minecraft consumed a key release.
    const acknowledged=[],unconfirmed=[];
    for(const slot of slots) {
      let confirmed=false;
      try {
        const receipt=await aiInput(config.executable,{
          slot,keys:0,dx:0,dy:0,ttlMs:1,requestedAtMs:Date.now(),
        },{writerId,mode:config.mode,leaseEpoch});
        confirmed=receipt?.schema===1&&receipt?.slot===slot&&
          receipt?.status==='NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED'&&
          receipt?.gameplayObserved===false;
      } catch { /* Continue all slots; never retry an uncertain packet. */ }
      (confirmed?acknowledged:unconfirmed).push(slot);
    }
    return {acknowledged,unconfirmed};
  }
  // An exited/replaced game may invalidate a lease AFTER a neutral ACK.
  // The native actor rejects a group release atomically. One attempted
  // release per confirmed-neutral slot prevents an unrelated client from
  // blocking its healthy peers. Never retry an uncertain release or
  // substitute another PID/birth/epoch.
  async function releaseAcknowledgedAiLeases(slots,leaseEpoch) {
    const released=[],unconfirmed=[];
    for(const slot of slots) {
      try {
        await changeAiLease('release',[slot],leaseEpoch);
        released.push(slot);
      } catch {
        unconfirmed.push(slot);
      }
    }
    return {released,unconfirmed};
  }
  function clearReconciledAiSlots(slots) {
    // Only original-channel neutral ACK + matching writer lease release
    // may clear the local MCP hold; no group-wide optimistic clearance.
    for(const slot of slots) {
      uncertainAiSlots.delete(slot);
      startupReconcileSlots.delete(slot);
    }
  }
  async function callTool(name,args) {
    requireOpen();
    if(name==='probe_qa' && args && Object.hasOwn(args,'inspectRun')) {
      // No preparation, source ZIP, log, client capture, status write,
      // native game input, owner transfer or controller startup.
      keysExactly(args,['slot','inspectRun']);
      if(args.inspectRun!==true)throw new Error('Invalid tool arguments');
      const slot=slotOf(args.slot??2);
      if(!config.executable)return {
        schema:1,slot,status:'TESTING_CONTROLLER_NOT_CONFIGURED',
        operatorNextAction:'REVIEW_NATIVE_CONTROLLER_SETUP',
        gameplayAcceptance:'NOT_VERIFIED',
        commandSentByLazyTester:false,
      };
      let inspection;
      try {
        inspection=await inspectTesting(config.executable,{
          slot,ownerId:testingOwnerId});
      }catch{
        return {schema:1,slot,status:'TESTING_INSPECTION_UNAVAILABLE',
          operatorNextAction:'REVIEW_NATIVE_CONTROLLER_AVAILABILITY',
          gameplayAcceptance:'NOT_VERIFIED',
          commandSentByLazyTester:false,
        };
      }
      // The original controller owns the run; the original Data/Testing
      // store owns committed summaries. Correlate read-only and return only
      // bounded source-only facts. Never create, finalize, retry or transfer.
      let archiveReview={schema:1,slot,status:'NOT_CONFIGURED',
        trust:'LOCAL_CHECKSUM_ONLY_NOT_MINECRAFT_OR_OPERATOR_ATTESTATION',
        authorizesTransfer:false};
      if(config.testingArchiveDirectory){
        try {
          archiveReview=await reviewTestingArchive({
            store:config.testingArchiveDirectory,slot,
            phase:inspection.phase,runId:inspection.runId,
            reportedStatus:inspection.reportedStatus,
            reportDigest:inspection.reportDigest,
          });
          if(archiveReview?.schema!==1||archiveReview.slot!==slot||
             archiveReview.authorizesTransfer!==false||
             archiveReview.trust!==
               'LOCAL_CHECKSUM_ONLY_NOT_MINECRAFT_OR_OPERATOR_ATTESTATION')
            throw new Error('Untrusted Testing archive review');
        }catch{
          archiveReview={schema:1,slot,
            status:'INTEGRITY_UNAVAILABLE_OR_CONFLICT',
            trust:'LOCAL_CHECKSUM_ONLY_NOT_MINECRAFT_OR_OPERATOR_ATTESTATION',
            authorizesTransfer:false};
        }
      }
      return {schema:1,slot,status:'TESTING_RUN_INSPECTION',
        inspection,archiveReview,
        operatorNextAction:testingRecoveryNextAction(inspection,archiveReview),
        gameplayAcceptance:'NOT_VERIFIED',
        commandSentByLazyTester:false,
      };
    }
    if(name==='probe_qa' && args && Object.hasOwn(args,'archivedRunId')) {
      // Historical summary is an explicit READ, never a new checkpoint,
      // test command, reservation, native reconnect or status publication.
      keysExactly(args,['archivedRunId']);
      if(typeof args.archivedRunId!=='string'||
         !/^[a-f0-9]{32}$/.test(args.archivedRunId))
        throw new Error('Invalid tool arguments');
      if(!config.testingArchiveDirectory)return {
        schema:1,status:'TESTING_ARCHIVE_NOT_CONFIGURED',
        gameplayAcceptance:'NOT_VERIFIED'};
      try {
        const result=await readArchivedTesting({
          store:config.testingArchiveDirectory,runId:args.archivedRunId});
        return {...result,
          operatorNextAction:'REVIEW_HISTORICAL_SOURCE_ONLY_RESULT'};
      } catch {
        return {schema:1,status:'TESTING_ARCHIVE_UNAVAILABLE_OR_INVALID',
          runId:args.archivedRunId,
          operatorNextAction:'REVIEW_ARCHIVE_INTEGRITY_MANUALLY',
          gameplayAcceptance:'NOT_VERIFIED'};
      }
    }
    // Opt-in AI convenience path over the SAME original Probe checkpoint.
    // Legacy calls without the flag are byte-for-byte behavior-compatible.
    // Desktop delivery is only a short-lived presentation mirror, never
    // another result store, runner, approval or proof of gameplay input.
    if(name==='probe_qa' && args &&
       (Object.hasOwn(args,'publishToDesktop') ||
        Object.hasOwn(args,'coordinateNative'))) {
      keysExactly(args,['slot','restart','scope','publishToDesktop','coordinateNative']);
      if(args.publishToDesktop!==undefined&&typeof args.publishToDesktop!=='boolean'||
         args.coordinateNative!==undefined&&typeof args.coordinateNative!=='boolean')
        throw new Error('Invalid tool arguments');
      const originalArguments={};
      for(const key of ['slot','restart','scope']) {
        if(Object.hasOwn(args,key))originalArguments[key]=args[key];
      }
      if(args.restart!==undefined&&typeof args.restart!=='boolean')
        throw new Error('Invalid tool arguments');
      if(args.scope!==undefined&&!['full','smoke','affected'].includes(args.scope))
        throw new Error('Invalid tool arguments');
      if(args.scope!==undefined&&!config.mapContractPath) {
        return {schema:1,slot:slotOf(args.slot??2),status:'BLOCKED',
          reason:'MAP_CONTRACT_NOT_CONFIGURED',
          operatorNextAction:'RESOLVE_APPROVED_PROBE_SOURCE_OR_CAPTURE',
          gameplayAcceptance:'NOT_VERIFIED'};
      }
      const slot=slotOf(args.slot??2);
      // Block human MC01 before any correlation reservation.
      if(config.mode==='human-plus-four-ai'&&slot===1)
        throw new Error('MC01 human host is protected from AI computer-use targeting');
      let runRef=null;
      let runId=null;
      if(args.coordinateNative===true) {
        if(!config.executable)return {
          schema:1,slot,status:'BLOCKED',
          reason:'TESTING_NATIVE_CONTROLLER_NOT_CONFIGURED',
          operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
          gameplayAcceptance:'NOT_VERIFIED',
          desktopDelivery:'NO_CURRENT_TESTING_CHECKPOINT_TO_MIRROR',
        };
        const existing=probeQaPlans.get(slot);
        if(config.testingArchiveDirectory&&!testingRunIds.has(slot)&&
           !existing) {
          // A controller restart loses its volatile run book. A committed
          // earlier result cannot be assumed to authorize a NEW run.
          // No automatic takeover/replay from a historical archive.
          let historical;
          try {
            historical=await findArchivedTestingSlot({
              store:config.testingArchiveDirectory,slot});
          } catch {
            return {schema:1,slot,status:'BLOCKED',
              reason:'TESTING_ARCHIVE_RECONCILIATION_REQUIRED',
              operatorNextAction:'REVIEW_ARCHIVE_INTEGRITY_MANUALLY',
              gameplayAcceptance:'NOT_VERIFIED'};
          }
          if(historical)return {schema:1,slot,status:'BLOCKED',
            reason:'TESTING_PRIOR_ARCHIVED_RUN_REQUIRES_RECONCILIATION',
            existingRunId:historical.runId,runPhase:'HISTORICAL_ARCHIVE',
            operatorNextAction:'READ_HISTORICAL_RESULT_NO_AUTOMATIC_RESTART',
            gameplayAcceptance:'NOT_VERIFIED'};
        }
        if(existing&&(!testingRunIds.has(slot)||
            existing.nativeRunId!==testingRunIds.get(slot)))return {
          schema:1,slot,status:'BLOCKED',
          reason:'EXISTING_LEGACY_TESTING_CHECKPOINT_NOT_COORDINATED',
          operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
          gameplayAcceptance:'NOT_VERIFIED',
        };
        if(args.restart===true&&!testingRunIds.has(slot))return {
          schema:1,slot,status:'BLOCKED',
          reason:'TESTING_RESTART_REQUIRES_OWNED_RUN_ID',
          operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
          gameplayAcceptance:'NOT_VERIFIED',
        };
        const previous=testingRunIds.get(slot)??null;
        runId=previous&&args.restart!==true?
          previous:randomBytes(16).toString('hex');
        try {
          runRef=await coordinateTesting(config.executable,{
            operation:'reserve',slot,runId,ownerId:testingOwnerId,
            replaceRunId:args.restart===true?previous:null,
          });
        } catch {
          return {schema:1,slot,status:'BLOCKED',
            reason:'TESTING_NATIVE_RESERVATION_UNCONFIRMED',
            operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
            gameplayAcceptance:'NOT_VERIFIED',
          };
        }
        if(runRef.runId!==runId||runRef.ownerMatch!==true) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'TESTING_RUN_ALREADY_RESERVED',
            existingRunId:runRef.runId,runPhase:runRef.phase,
            recoveryAction:runRef.phase==='FINAL'?
              'READ_ARCHIVED_RESULT_IF_CONFIGURED':
              'VERIFY_PREVIOUS_OWNER_NO_AUTOMATIC_TAKEOVER',
            operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
            gameplayAcceptance:'NOT_VERIFIED',
          };
        }
        if(runRef.phase==='FINAL' && args.restart!==true &&
            !existing?.terminalReport) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'TESTING_FINAL_REFERENCE_EXISTS_LOCAL_REPORT_UNAVAILABLE',
            testingRunId:runRef.runId,
            reportedStatus:runRef.reportedStatus,
            reportDigest:runRef.reportDigest,
            operatorNextAction:'REVIEW_BLOCKED_OR_INCOMPLETE_TERMINAL_REPORT',
            gameplayAcceptance:'NOT_VERIFIED',
          };
        }
      }
      const result=await callTool('probe_qa',originalArguments);
      const operatorNextAction=operatorNextTestingAction(result);
      if(runRef!==null) {
        const actualPlan=probeQaPlans.get(slot);
        if(actualPlan?.nativeRunId&&actualPlan.nativeRunId!==runId) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'TESTING_CANONICAL_RUN_REPLACED_DURING_READ',
            operatorNextAction:'RESOLVE_BLOCKER_NO_AUTOMATIC_RESTART',
            gameplayAcceptance:'NOT_VERIFIED'};
        }
        // Store only the correlation ID on the existing Probe checkpoint;
        // native actor remains the sole cross-MCP admission authority.
        if(actualPlan)actualPlan.nativeRunId=runId;
        testingRunIds.set(slot,runId);
      }
      let terminalReference=null;
      if(runRef!==null) {
        if(result.terminal===true||result.phase==='PREFLIGHT') {
          const outcome=['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW',
            'INCOMPLETE','BLOCKED'].includes(result.status)?
            result.status:'BLOCKED';
          const reportDigest=createHash('sha256')
            .update(JSON.stringify(result)).digest('hex');
          try {
            terminalReference=await coordinateTesting(config.executable,{
              operation:'finalize',slot,runId,ownerId:testingOwnerId,
              reportedStatus:outcome,reportDigest,
            });
          } catch {
            return {...result,operatorNextAction,testingRunId:runId,
              nativeRunStatus:'FINAL_REFERENCE_UNCONFIRMED',
              desktopDelivery:'STATUS_DELIVERY_UNAVAILABLE'};
          }
        }
      }
      let archiveStatus=null;
      if(runRef!==null && terminalReference?.phase==='FINAL' &&
         result.terminal===true) {
        if(!config.testingArchiveDirectory) {
          archiveStatus='NOT_CONFIGURED';
        } else {
          try {
            const saved=await archiveTesting({
              store:config.testingArchiveDirectory,
              runId,slot,report:result,
              reportDigest:terminalReference.reportDigest});
            if(saved?.status!=='ARCHIVED_SOURCE_ONLY'||
               saved.runId!==runId||saved.slot!==slot||
               saved.originalDigest!==terminalReference.reportDigest)
              throw new Error('Testing archive receipt invalid');
            archiveStatus='ARCHIVED_SOURCE_ONLY';
          }catch {
            // Controller may already be FINAL; report the failure without
            // replaying the source or treating the digest as a file.
            archiveStatus='ARCHIVE_UNAVAILABLE_OR_CONFLICT';
          }
        }
      }
      const shaped={...result,operatorNextAction,
        ...(archiveStatus!==null?{archiveStatus,
          ...(archiveStatus==='ARCHIVED_SOURCE_ONLY'?{archivedRunId:runId}:{})}:{}),
        ...(runRef!==null?{
          testingRunId:runId,
          nativeRunStatus:terminalReference?.phase??runRef.phase,
          ...(terminalReference?{
            reportDigest:terminalReference.reportDigest}:{}),
        }:{})};
      if(args.publishToDesktop!==true)return shaped;
      if(result.phase==='PREFLIGHT'||
         result.reason==='QA_PLAN_SUPERSEDED'||
         !probeQaPlans.has(slot)) {
        return {...shaped,
          desktopDelivery:'NO_CURRENT_TESTING_CHECKPOINT_TO_MIRROR'};
      }
      const mirrored=await callTool('agent_capabilities',
        {publishToDesktop:true});
      return {...shaped,
        desktopDelivery:mirrored.desktopDelivery??'STATUS_DELIVERY_UNAVAILABLE'};
    }
    if(name==='probe_qa') {
      keysExactly(args,['slot','restart','scope']);
      const slot=slotOf(args.slot??2);
      if(args.scope!==undefined&&!['full','smoke','affected'].includes(args.scope)) {
        throw new Error('Invalid tool arguments');
      }
      if(args.scope!==undefined&&!config.mapContractPath) {
        return {schema:1,slot,status:'BLOCKED',
          reason:'MAP_CONTRACT_NOT_CONFIGURED',gameplayAcceptance:'NOT_VERIFIED'};
      }
      if(args.restart!==undefined&&typeof args.restart!=='boolean') {
        throw new Error('Invalid tool arguments');
      }
      if(config.mode==='human-plus-four-ai'&&slot===1) {
        throw new Error('MC01 human host is protected from AI computer-use targeting');
      }
      if(probeQaBusy.has(slot))return {schema:1,slot,status:'BLOCKED',
        reason:'QA_OPERATION_IN_PROGRESS',gameplayAcceptance:'NOT_VERIFIED'};
      probeQaBusy.add(slot);
      try {
        let plan=probeQaPlans.get(slot);
        if(plan&&args.restart!==true&&args.scope!==undefined&&
           args.scope!==plan.mapScope) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'QA_SCOPE_CHANGE_REQUIRES_RESTART',gameplayAcceptance:'NOT_VERIFIED'};
        }
        if(plan&&args.restart!==true&&config.mapContractPath&&
           !plan.mapSourceRevision) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'MAP_PLAN_RESTART_REQUIRED',gameplayAcceptance:'NOT_VERIFIED'};
        }
        // A completed or stopped run is a historical snapshot, not new
        // game readiness. Only explicit restart creates a new checkpoint.
        if(plan?.terminalReport&&args.restart!==true) {
          return plan.terminalReport;
        }
        // One user-facing tool. Existing prepare/read tools remain for
        // backward-compatible callers, but AI need not carry a session ID.
        if(!plan||args.restart===true) {
          let mapPlan=null,mapSourceRevision=null;
          const mapScope=args.scope??'full';
          if(config.mapContractPath) {
            try {
              const loaded=await mapContract(config.mapContractPath);
              const planned=compactMapTestPlan(
                planMapTests(loaded.contract,{scope:mapScope}));
              // Byte fingerprint is a historical source binding only,
              // never an approval signature or proof of a live game.
              mapPlan=Object.freeze({...planned,
                ...(typeof loaded.contentSha256==='string'&&
                  /^[a-f0-9]{64}$/.test(loaded.contentSha256)?
                    {contractSha256:loaded.contentSha256}:{})});
              mapSourceRevision=loaded.sourceRevision;
            } catch {
              return {schema:1,slot,phase:'PREFLIGHT',status:'BLOCKED',
                reason:'MAP_CONTRACT_REVIEW_REQUIRED',
                commandSentByLazyTester:false,gameplayAcceptance:'NOT_VERIFIED'};
            }
          }
          let prepared;
          try {prepared=await callTool('prepare_probe_qa',{slot});}
          catch(error) {
            const reason=error?.message==='PROBE_SOURCE_REVIEW_REQUIRED'?
              'PROBE_SOURCE_REVIEW_REQUIRED':'PROBE_SOURCE_OR_CAPTURE_UNAVAILABLE';
            return {schema:1,slot,phase:'PREFLIGHT',status:'BLOCKED',
              reason,commandSentByLazyTester:false,
              gameplayAcceptance:'NOT_VERIFIED',
              next:'Check the approved ZIP and local capture configuration; do not dispatch a command.',
              ...(mapPlan?{mapPlan,qaReport:composeMapQaReport(mapPlan,
                {schema:1,status:'BLOCKED',reason},probeQaProofGate())}:{})};
          }
          if(prepared.status!=='PREPARED_SOURCE_ONLY') {
            return {schema:1,slot,phase:'PREFLIGHT',status:'BLOCKED',
              reason:prepared.status,commandSentByLazyTester:false,
              gameplayAcceptance:'NOT_VERIFIED',
              next:prepared.status==='CAPTURE_SLOT_COLLISION'?
                'MC slots must use distinct original capture files; verify collection provenance before a new QA checkpoint.':
                'Configure the approved ZIP and stable capture once; retry probe_qa.',
              ...(mapPlan?{mapPlan,qaReport:composeMapQaReport(mapPlan,
                {schema:1,status:'BLOCKED',reason:prepared.status},
                probeQaProofGate())}:{})};
          }
          const activePlan=probeQaPlans.get(slot);
          if(activePlan?.sessionId!==prepared.sessionId) {
            return {schema:1,slot,status:'BLOCKED',
              reason:'QA_PLAN_SUPERSEDED',gameplayAcceptance:'NOT_VERIFIED'};
          }
          activePlan.mapScope=mapScope;
          if(mapPlan) {
            activePlan.mapPlan=Object.freeze(mapPlan);
            activePlan.mapSourceRevision=mapSourceRevision;
            let fresh;
            try {fresh=await mapRevisionStatus(config.mapContractPath,mapSourceRevision);}
            catch {fresh='UNAVAILABLE';}
            if(fresh!=='UNCHANGED') {
              return settleProbeQa(slot,activePlan,{schema:1,slot,status:'BLOCKED',
                reason:fresh==='CHANGED'?'MAP_CONTRACT_CHANGED_STOP':
                  'MAP_CONTRACT_UNAVAILABLE',gameplayAcceptance:'NOT_VERIFIED'});
            }
          }
          let targetStatus='NATIVE_TARGET_NOT_CONFIGURED';
          let targetHandoff=null;
          // Reuse the existing strict PID/HWND/birth gate. A source-only
          // plan never authorizes keyboard input or infers a loaded world.
          if(config.executable) {
            targetStatus='NATIVE_TARGET_UNAVAILABLE';
            try {
              const target=await callTool('get_computer_use_target',{slot});
              if(target?.verification===
                 'NATIVE_PROCESS_AND_WINDOW_VERIFIED_AT_READ_TIME'&&
                 target.identity?.pid>0&&target.identity?.windowId>0&&
                 /^\d{1,20}$/.test(target.identity.createdFileTime??'')) {
                const active=probeQaPlans.get(slot);
                if(active?.sessionId===prepared.sessionId) {
                  active.targetIdentity={
                    pid:target.identity.pid,
                    windowId:target.identity.windowId,
                    createdFileTime:target.identity.createdFileTime};
                  targetHandoff={
                    window:{kind:'window',pid:target.identity.pid,
                      window_id:target.identity.windowId},
                    identity:{...active.targetIdentity,
                      sessionId:target.identity.sessionId},
                    verification:'PROCESS_WINDOW_ONLY_NOT_GAME_WORLD',
                    input:'NOT_AVAILABLE_OR_AUTHORIZED'};
                  targetStatus='NATIVE_TARGET_OBSERVED_NOT_GAME_READY';
                }
              }
            } catch { /* Native read-only target may be unavailable. */ }
          }
          if(probeQaPlans.get(slot)?.sessionId!==prepared.sessionId) {
            return {schema:1,slot,status:'BLOCKED',
              reason:'QA_PLAN_SUPERSEDED',gameplayAcceptance:'NOT_VERIFIED'};
          }
          // The gate is a source-only readiness report, never a caller
          // claim of Player/admin, world or live debugger provenance.
          const proofGate=probeQaProofGate({
            sourceReady:true,captureReady:true,
            targetBound:targetHandoff!==null,
          });
          activePlan.proofGate=proofGate;
          return {schema:1,slot,phase:'AWAITING_AUTHORIZED_COMMAND',
            status:'BLOCKED',reason:'COMMAND_DISPATCH_NOT_ENABLED',
            sourceCapture:'APPROVED_SOURCE_LOG_FILE_PRESENT',gameExecution:'NOT_VERIFIED',
            probeCount:prepared.autoProbeCount,target:targetStatus,
            ...(targetHandoff?{targetHandoff}:{}),
            command:prepared.command,
            requires:'AUTHORIZED_PLAYER_ADMIN_AND_QA_WORLD',
            commandSentByLazyTester:false,gameplayAcceptance:'NOT_VERIFIED',
            proofGate,
            ...(mapPlan?{mapPlan,qaReport:composeMapQaReport(mapPlan,
              {schema:1,status:'BLOCKED',
                reason:'COMMAND_DISPATCH_NOT_ENABLED'},proofGate)}:{})};
        }
        // Do NOT read/hash/parse the ZIP again on every result query.
        // Only compare cheap filesystem change tokens against the pinned
        // source revision. A changed ZIP is reviewed on NEXT prepare.
        let revision;
        try {revision=await probeRevisionStatus(config.probeZip,plan.sourceRevision);}
        catch {revision='UNAVAILABLE';}
        if(revision!=='UNCHANGED') {
          return settleProbeQa(slot,plan,{schema:1,slot,status:'BLOCKED',
            reason:revision==='CHANGED'?'QA_SOURCE_CHANGED_STOP':
              'QA_SOURCE_UNAVAILABLE',gameplayAcceptance:'NOT_VERIFIED'});
        }
        if(plan.mapSourceRevision&&config.mapContractPath) {
          let sourceStatus;
          try {sourceStatus=await mapRevisionStatus(config.mapContractPath,
            plan.mapSourceRevision);}catch{sourceStatus='UNAVAILABLE';}
          if(sourceStatus!=='UNCHANGED') {
            return settleProbeQa(slot,plan,{schema:1,slot,status:'BLOCKED',
              reason:sourceStatus==='CHANGED'?'MAP_CONTRACT_CHANGED_STOP':
                'MAP_CONTRACT_UNAVAILABLE',gameplayAcceptance:'NOT_VERIFIED'});
          }
        }
        let raw;
        try {
          raw=await callTool('read_probe_qa_result',{
            slot,sessionId:plan.sessionId});
        } catch {
          return {schema:1,slot,status:'INCOMPLETE',
            reason:'EVIDENCE_UNAVAILABLE',gameplayAcceptance:'NOT_VERIFIED'};
        }
        let compact=compactProbeQaResult(raw,slot);
        // Waiting/incomplete evidence needs no native IPC or window probe.
        // Revalidate only when there is an actual Probe outcome to report.
        if(probeQaPlans.get(slot)?.sessionId!==plan.sessionId) {
          return {schema:1,slot,status:'BLOCKED',
            reason:'QA_PLAN_SUPERSEDED',gameplayAcceptance:'NOT_VERIFIED'};
        }
        if(raw.status==='QA_PLAN_EXPIRED') {
          return settleProbeQa(slot,plan,compact);
        }
        // Runner markers that are missing, duplicated, suppressed or
        // inconsistent cannot be repaired by rereading the same bytes.
        // A new run (restart=true) is required for fresh provenance.
        if(/^INCOMPLETE_[A-Z_]+$/.test(raw.status??'')) {
          return settleProbeQa(slot,plan,compact);
        }
        // A rotated/replaced/truncated or lost Probe capture boundary
        // cannot be repaired within this session. A torn concurrent read
        // may recover, so only these explicit reasons are terminal.
        if(raw.status==='CAPTURE_FILE_UNSTABLE'&&
           ['PROBE_CAPTURE_REPLACED','PROBE_CAPTURE_TRUNCATED',
            'PROBE_CAPTURE_TAIL_GAP','PROBE_CAPTURE_PREFIX_CHANGED',
            'PROBE_MARKER_VOLUME_EXCEEDED'].includes(raw.captureIssue)) {
          return settleProbeQa(slot,plan,compact);
        }
        if(['WAITING','INCOMPLETE','BLOCKED'].includes(compact.status)) {
          return compact;
        }
        let target='NOT_PINNED_AT_PREPARE';
        if(plan.targetIdentity) {
          target='CHANGED_OR_UNAVAILABLE';
          try {
            const checked=await callTool('revalidate_computer_use_target',{
              slot,...plan.targetIdentity});
            if(checked?.matched===true) {
              target='NATIVE_PROCESS_MATCH_AT_READ_TIME_ONLY';
            }
          } catch { /* No game input or inferred fallback. */ }
        }
        // Never display a report from a superseded concurrent plan.
        if(probeQaPlans.get(slot)?.sessionId!==plan.sessionId&&
           raw.status!=='QA_PLAN_EXPIRED') {
          return {schema:1,slot,status:'BLOCKED',
            reason:'QA_PLAN_SUPERSEDED',gameplayAcceptance:'NOT_VERIFIED'};
        }
        if(plan.targetIdentity&&target==='CHANGED_OR_UNAVAILABLE') {
          compact={...compact,status:'INCOMPLETE',
            reason:'QA_NATIVE_TARGET_CHANGED',
            ...(plan.mapPlan?{probeReportedStatus:compact.status}:{}),
            gameplayAcceptance:'NOT_VERIFIED'};
        } else if(!plan.targetIdentity) {
          // Preserve the original Probe's limited reported outcome for
          // troubleshooting without presenting it as map/world acceptance.
          compact={...compact,status:'INCOMPLETE',
            reason:'QA_NATIVE_TARGET_NOT_BOUND',
            ...(plan.mapPlan?{probeReportedStatus:compact.status}:{}),
            gameplayAcceptance:'NOT_VERIFIED'};
        }
        return settleProbeQa(slot,plan,{...compact,target});
      } finally {probeQaBusy.delete(slot);}
    }
    if(name==='prepare_probe_qa') {
      keysExactly(args,['slot']);
      const slot=slotOf(args.slot??2);
      if(config.mode==='human-plus-four-ai'&&slot===1) {
        throw new Error('MC01 human host is protected from AI computer-use targeting');
      }
      // Preparing again invalidates any older result token for the same slot.
      // A direct legacy prepare cannot inherit a previous native run ID.
      // The coordinated wrapper reattaches its admitted ID to the new
      // canonical plan after the ONE original probe_qa call completes.
      probeQaPlans.delete(slot);
      testingRunIds.delete(slot);
      if(!config.probeZip)return {schema:1,slot,status:'PROBE_ZIP_NOT_CONFIGURED',
        needed:'Set LAZY_TESTER_PROBE_ZIP once to the approved local Skills Probe ZIP',
        commandSent:false};
      if(!config.evidenceDirectory)return {schema:1,slot,status:'CAPTURE_NOT_CONFIGURED',
        needed:'Configure the existing per-profile evidence directory once',
        commandSent:false};
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      const catalog=await probeCatalog(config.probeZip);
      if(!catalog||!Array.isArray(catalog.autoIds)||
        catalog.autoIds.length<1||catalog.autoIds.length>80||
        catalog.autoCount!==catalog.autoIds.length||
        !/^[a-f0-9]{64}$/.test(catalog.sourceRevision??'')||
        new Set(catalog.autoIds).size!==catalog.autoIds.length||
        catalog.autoIds.some(id=>typeof id!=='string'||
          !/^P[0-9]{1,3}[A-Z0-9]{0,12}$/.test(id))||
        !/^[a-f0-9]{64}$/.test(catalog.archiveSha256??'')||
        !/^[a-f0-9]{64}$/.test(catalog.runnerSha256??'')) {
        throw new Error('PROBE_SOURCE_REVIEW_REQUIRED');
      }
      const existing=await evidence(config.evidenceDirectory,slot,EVIDENCE_LIMIT,{
        probeOnly:true});
      if(existing.status!=='CAPTURE_FILE_PRESENT') {
        return {schema:1,slot,status:'CAPTURE_NOT_READY',
          captureStatus:existing.status,commandSent:false};
      }
      if(existing.captureCursor?.kind!=='STABLE_CAPTURE_CURSOR_V1') {
        return {schema:1,slot,status:'CAPTURE_CHECKPOINT_UNAVAILABLE',
          commandSent:false,gameplayAcceptance:'NOT_VERIFIED'};
      }
      // Two supposedly independent MC slots must never accept the SAME
      // underlying log inode as independent probe evidence. This detects
      // a misconfigured hard-link/shared capture, not world authenticity.
      const captureFileIdentity=existing.captureCursor.fileIdentity;
      if(typeof captureFileIdentity==='string'&&
         /^[a-f0-9]{64}$/.test(captureFileIdentity)&&
         [...probeQaPlans.entries()].some(([otherSlot,other])=>
           otherSlot!==slot&&
           other.captureCursor?.fileIdentity===captureFileIdentity)) {
        return {schema:1,slot,status:'CAPTURE_SLOT_COLLISION',
          commandSent:false,gameplayAcceptance:'NOT_VERIFIED'};
      }
      const sessionId=randomBytes(12).toString('hex');
      const preparedAt=new Date(qaClock()).toISOString();
      probeQaPlans.set(slot,{sessionId,slot,preparedAt,autoIds:catalog.autoIds,
        sourceRevision:catalog.sourceRevision,
        captureCursor:existing.captureCursor});
      return {schema:1,slot,status:'PREPARED_SOURCE_ONLY',
        sessionId,preparedAt,autoProbeCount:catalog.autoCount,
        autoProbeIds:catalog.autoIds,
        catalogCache:catalog.cache,
        sourceZipSha256:catalog.archiveSha256,
        command:'/scriptevent probe:run auto',
        dispatch:'NOT_AVAILABLE_IN_LAZY_TESTER',
        computerUse:{input:'NOT_VERIFIED',approvedWorldRequired:true,
          playerAdminRequired:true,requiresExactTarget:true,
          existingTargetTool:'get_computer_use_target',
          revalidationTool:'revalidate_computer_use_target',
          commandInputTool:'NOT_AVAILABLE'},
        requirements:'Copied QA world; Skills Probe installed; approved Player/admin; confirmed target profile and command-entry capability',
        next:'After an authorized operator or verified AI Computer Use executes the original command, call read_probe_qa_result for this sessionId.',
        commandSent:false,gameplayAcceptance:'NOT_VERIFIED'};
    }
    if(name==='read_probe_qa_result') {
      keysExactly(args,['slot','sessionId']);
      const slot=slotOf(args.slot);
      if(typeof args.sessionId!=='string'||!/^[a-f0-9]{24}$/.test(args.sessionId)) {
        throw new Error('Invalid tool arguments');
      }
      const session=probeQaPlans.get(slot);
      if(!session||session.sessionId!==args.sessionId) {
        return {schema:1,slot,status:'NOT_PREPARED_OR_EXPIRED',
          gameplayAcceptance:'NOT_VERIFIED'};
      }
      // Do not reuse an old source plan indefinitely; no timer/worker needed.
      if(session.expired===true||
         qaClock()-Date.parse(session.preparedAt)>20*60*1000) {
        // Retain an expired tombstone so the preferred one-entry tool
        // cannot silently start a new QA run on the next poll.
        session.expired=true;
        return {schema:1,slot,status:'QA_PLAN_EXPIRED',
          gameplayAcceptance:'NOT_VERIFIED'};
      }
      const report=await evidence(config.evidenceDirectory,slot,EVIDENCE_LIMIT,{
        probeCheckpoint:session.captureCursor});
      if(probeQaPlans.get(slot)?.sessionId!==session.sessionId) {
        return {schema:1,slot,status:'QA_PLAN_SUPERSEDED',
          gameplayAcceptance:'NOT_VERIFIED'};
      }
      return {slot,sessionId:session.sessionId,
        ...reviewProbeRun(report.probeEvents??[],session,{
          captureStatus:report.status,captureReason:report.reason,
          captureTruncated:report.truncated})};
    }
    // All diagnostic calls are read-only, scoped to one configured Data/
    // Diagnostics root. A Codex/MCP caller cannot import or request a path.
    if(name==='get_diagnostics_overview') {
      keysExactly(args,['run']);
      return diagnosticsOverview(config.diagnosticsDirectory,args.run);
    }
    if(name==='list_issue_candidates') {
      keysExactly(args,['run','cursor','limit']);
      return listDiagnosticIssues(config.diagnosticsDirectory,args.run,
        {cursor:args.cursor,limit:args.limit});
    }
    if(name==='get_issue_details') {
      keysExactly(args,['run','id']);
      return getDiagnosticIssue(config.diagnosticsDirectory,args.run,args.id);
    }
    if(name==='query_diagnostic_evidence') {
      keysExactly(args,['run','id','cursor','limit']);
      return queryDiagnosticEvidence(config.diagnosticsDirectory,args.run,args.id,
        {cursor:args.cursor,limit:args.limit});
    }
    if (name==='start_visual_observation') {
      keysExactly(args,['slots','intervalMs','durationMs']);
      const slots=checkedSlots(args.slots);
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
      inFlight=true;
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
          await gpuSource.start(slot,target,{durationMs,fps});
          started.push(slot);
          if (startup.cancelled) throw new Error('GPU_OBSERVATION_START_CANCELLED');
        }
        const response=visualObserver.start(slots,{intervalMs,durationMs});
        return {...response,source:'PERSISTENT_NATIVE_WGC_EXPLICIT_OPT_IN',
          pixelsPassedToModel:'ONLY_ON_MCP_READ'};
      } catch(error) {
        gpuSource.stop(started);
        throw error;
      } finally {
        if (gpuStartup===startup) gpuStartup=null;
        inFlight=false;
      }
    }
    if (name==='get_multi_client_visual_state') {
      keysExactly(args,['slots','since']);
      const slots=checkedSlots(args.slots ??
        (config.mode==='human-plus-four-ai'?[2,3,4,5]:[1,2,3,4,5]));
      // Local cache read only: do NOT trigger new native inspections,
      // create GPU workers, or silently hand image bytes to the model.
      const readings=slots.map(slot=>visualObserver.read(slot,{
        includeImage:false,
      }).metadata);
      const visual=summarizeTeamVisualState(
        config.mode,slots,readings,args.since??[]);
      const state=summarizeTeamCoordination(
        config.mode,visual,[...uncertainAiSlots].sort((a,b)=>a-b),multiEnabled,
        [...startupReconcileSlots].sort((a,b)=>a-b));
      // A source-ready cache must not advertise a new action opportunity
      // while the original controller/observer MCP lock is still active.
      const busy=exclusiveBusy();
      return {...visual,coordination:{
        ...state,observedActionEligible:state.observedActionEligible&&!busy,
        status:busy&&!state.blockedSlots.length?
          'MCP_OPERATION_IN_PROGRESS':state.status,
      }};
    }
    if (name==='read_visual_observation') {
      keysExactly(args,['slot','afterSequence','includeImage','screenReview']);
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
           b.producerSequence!==a.producerSequence||
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
      } else if(wantsImage) {
        // No new PNG requested or no usable cached frame: retain the
        // cheap metadata-only behavior without probing native windows.
        observation=visualObserver.read(slot,{afterSequence,includeImage:true});
      }
      // This remains an ephemeral cache READ, not a screenshot, game input,
      // player/world attribution or automatic clipboard operation.
      const metadata={...observation.metadata,
        f3VisualReview:projectF3VisualReview(observation.metadata)};
      if(args.screenReview!==undefined){
        metadata.f3ScreenXYZ=reviewVisibleXYZFromFrame(metadata,args.screenReview);
      }
      return {...observation,metadata};
    }
    if (name==='stop_visual_observation') {
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
    }
    if (name==='assess_client_background_action') {
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
    }
    if (name==='inspect_client_ui') {
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
        const ui=await inspectCua(config.cuaExecutable,before);
        const after=await readNative('target',slot);
        requireOpen();
        if(!sameNativeVisualTarget(after,before)) {
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        return {...ui,slot,identity:{pid:after.pid,windowId:after.windowId,
          createdFileTime:after.createdFileTime,sessionId:after.sessionId}};
      } finally {visualReadBusy=false;}
    }
    if (name==='observe_client') {
      keysExactly(args,['slot']);
      return captureVerifiedFrame(slotOf(args.slot));
    }
    if (name==='get_computer_use_target' ||
        name==='revalidate_computer_use_target') {
      keysExactly(args,name==='get_computer_use_target' ?
        ['slot'] : ['slot','pid','windowId','createdFileTime']);
      const slot=slotOf(args.slot);
      if (config.mode==='human-plus-four-ai' && slot===1) {
        throw new Error('MC01 human host is protected from AI computer-use targeting');
      }
      if (name==='revalidate_computer_use_target' &&
          (!Number.isSafeInteger(args.pid) || args.pid<=0 ||
           !Number.isSafeInteger(args.windowId) || args.windowId<=0 ||
           typeof args.createdFileTime!=='string' ||
           !/^\d{1,20}$/.test(args.createdFileTime))) {
        throw new Error('Expected window identity must be exact and numeric');
      }
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        const current=await readNative('target',slot);
        if (name==='get_computer_use_target') {
          return computerUseTarget(current,slot,config.mode);
        }
        const matched=args.pid===current.pid &&
          args.windowId===current.windowId &&
          args.createdFileTime===current.createdFileTime;
        return {
          schema:1,slot,matched,
          status:matched?'IDENTITY_VERIFIED_AT_READ_TIME':'TARGET_CHANGED_REFUSE_ACTION',
          gameInput:'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED',
          note:'This is a fresh identity check, NOT a guarantee that an external tool will use this target or that Minecraft receives background input.',
        };
      } finally {inFlight=false;}
    }
    if (name==='get_test_context') {
      keysExactly(args,['slots']);
      const slots=normalizeEvidenceSlots(args.slots ?? [1,2,3,4,5]);
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        const nativeSnapshot=await readNative('inspect');
        const peerPlan=planPeerSession(config.mode,nativeSnapshot);
        const gameEvidence=await summarizeGameEvidence(
          config.evidenceDirectory,slots,evidence);
        return {
          schema:1,mode:config.mode,hostSlot:1,
          nativeProfiles:nativeSnapshot.profiles,
          peerPlan,gameEvidence,
          gameControl:'INDEPENDENT_BACKGROUND_INPUT_NOT_VERIFIED',
          contextScope:'NATIVE_SESSION_AND_UNATTRIBUTED_PROBE_EVIDENCE_ONLY',
          note:'Current process state is not proof of player login, world creation, join, or input acceptance.',
        };
      } finally {inFlight=false;}
    }
    if (name==='summarize_test_evidence') {
      keysExactly(args,['slots','window']);
      if(args.window===undefined) {
        return summarizeGameEvidence(config.evidenceDirectory,
          args.slots ?? [1,2,3,4,5],evidence);
      }
      if(!['begin','read'].includes(args.window)||!Object.hasOwn(args,'slots')) {
        throw new Error('Invalid tool arguments');
      }
      const slots=normalizeEvidenceSlots(args.slots);
      if(evidenceWindowBusy)throw new Error('Evidence window operation in progress');
      evidenceWindowBusy=true;
      try {
        const begin=args.window==='begin';
        // A failed explicit new baseline must not reuse an earlier test's
        // cursor. Repeated read stays anchored; no implicit rebase or retry.
        if(begin)for(const slot of slots)evidenceBaselines.delete(slot);
        const reports=await Promise.all(slots.map(async slot=>{
          const checkpoint=evidenceBaselines.get(slot);
          if(!begin&&!checkpoint) {
            return {slot,status:'UNAVAILABLE',events:[],baselineRequired:true};
          }
          try {
            return await evidence(config.evidenceDirectory,slot,EVIDENCE_LIMIT,
              begin?{probeOnly:true}:{evidenceCheckpoint:checkpoint});
          }catch {
            return {slot,status:'UNAVAILABLE',events:[]};
          }
        }));
        requireOpen();
        const windowSlots=slots.map((slot,index)=>{
          const report=reports[index];
          const present=report?.slot===slot&&report.status==='CAPTURE_FILE_PRESENT';
          if(begin&&present&&validCaptureCursor(report.captureCursor)) {
            evidenceBaselines.set(slot,Object.freeze({...report.captureCursor}));
            return {slot,status:'BASELINE_READY',
              baselinePartialLineExcluded:!report.captureCursor.completeLastLine};
          }
          if(!begin&&present&&Array.isArray(report.events)&&report.events.length<=EVIDENCE_LIMIT&&
             typeof report.windowCoverage?.recordLimitReached==='boolean'&&
             typeof report.windowCoverage?.baselinePartialLineExcluded==='boolean') {
            return {slot,status:'APPENDED_COMPLETE_LINES_ONLY',
              recordLimitReached:report.windowCoverage.recordLimitReached,
              baselinePartialLineExcluded:report.windowCoverage.baselinePartialLineExcluded,
              incompleteTrailingLineIgnored:report.incompleteTrailingLineIgnored===true};
          }
          const status=report?.slot===slot&&report.baselineRequired===true?'BASELINE_REQUIRED':
            report?.slot===slot&&['NOT_CONFIGURED','NO_CAPTURE_FILE','CAPTURE_FILE_UNSTABLE'].includes(report.status)?
              report.status:'UNAVAILABLE';
          // A definitive continuity failure permanently invalidates this
          // baseline. A transient live-write conflict may be read again.
          if(!begin&&report?.slot===slot&&(
             status==='NO_CAPTURE_FILE'||(status==='CAPTURE_FILE_UNSTABLE'&&
               ['EVIDENCE_CHECKPOINT_INVALID','EVIDENCE_CAPTURE_REPLACED',
                'EVIDENCE_CAPTURE_TRUNCATED','EVIDENCE_CAPTURE_TAIL_GAP',
                'EVIDENCE_CAPTURE_PREFIX_CHANGED','FILE_DISAPPEARED_DURING_READ'].includes(report.reason)))) {
            evidenceBaselines.delete(slot);
          }
          // Do not pass an invalid provider result to the summary projector.
          reports[index]={slot,status,events:[]};
          return {slot,status};
        });
        const captureWindow={mode:args.window,slots:windowSlots,
          scope:'LOCAL_CAPTURE_BYTES_NOT_VERIFIED_GAME_SESSION',
          timeline:'BOUNDED_SUMMARY_NOT_COMPLETE_GAMEPLAY_HISTORY'};
        if(begin)return {schema:1,captureWindow,
          verdict:'EVIDENCE_BASELINE_ONLY_NOT_GAMEPLAY_ACCEPTANCE'};
        return {...summarizeEvidenceReports(slots,reports),captureWindow};
      }finally {evidenceWindowBusy=false;}
    }
    if (name==='plan_player_actions') {
      keysExactly(args,['actions','groups','includeTimeline','includeClientFrames']);
      if (Object.hasOwn(args,'actions')===Object.hasOwn(args,'groups')||
          (args.includeTimeline!==undefined &&
           typeof args.includeTimeline!=='boolean') ||
          (args.includeClientFrames!==undefined &&
           typeof args.includeClientFrames!=='boolean') ||
          (args.includeTimeline===true && args.includeClientFrames===true)) {
        throw new Error('Invalid tool arguments');
      }
      const compact=Object.hasOwn(args,'groups');
      const actions=compact?expandActionGroups(args.groups):args.actions;
      if(args.includeClientFrames===true) {
        // Pure preview: NEVER contacts native controller or sends input.
        return compilePerClientInputFrames(config.mode,actions);
      }
      if(args.includeTimeline===true)return compilePlayerInputTimeline(config.mode,actions);
      const plan=planPlayerActions(config.mode,actions);
      // Old callers keep their detailed response. Compact authoring returns
      // only counts and the gate unless detailed preview is explicitly asked.
      if(!compact)return plan;
      return {schema:plan.schema,mode:plan.mode,
        slots:[...new Set(plan.actions.map(action=>action.slot))].sort((a,b)=>a-b),
        actionCount:plan.actions.length,
        horizonMs:Math.max(...plan.actions.map(action=>action.endMs)),
        maximumConcurrentIntents:plan.maximumConcurrentIntents,
        execution:plan.execution,gate:plan.gate};
    }
    if (name==='stop_experimental_multi_client_actions') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['slots']);
      // User may name slots in any order. The native atomic lease
      // contract needs ascending distinct slots, not UI request order.
      const slots=checkedSlots(args.slots).sort((a,b)=>a-b);
      if(multiSequence) {
        if(!slots.some(slot=>multiSequence.slots.includes(slot))) {
          throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
        }
        // One shared CLI schedule: partial cancellation is unsafe.
        // The native process is cancelled and ALL its touched slots get
        // independent neutral cleanup (also repeated by MCP finally).
        markAiUncertain(multiSequence.slots);
        multiSequence.abort.abort();
        return {schema:1,slots:multiSequence.slots,
          status:'CANCELLATION_REQUESTED_ALL_ACTIVE_AI_SLOTS',
          neutralRelease:'PENDING_SEQUENCE_COMPLETION_OR_NATIVE_TTL',
          gameplayObserved:false};
      }
      if(inFlight)throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      // A fresh epoch for THIS explicit neutral-recovery transaction.
      // It fences old CLI input packets from the same MCP writer.
      const leaseEpoch=makeAiLeaseEpoch();
      let leaseAttempted=false,leaseHeld=false;
      let released=[];
      try {
        // Claim the ENTIRE requested set in the original controller
        // before writing any neutral. Foreign writers are never touched.
        leaseAttempted=true;
        await changeAiLease('acquire',slots,leaseEpoch,true);
        leaseHeld=true;
        const neutral=await releaseAiSlots(slots,leaseEpoch);
        // Only clients with an actual native neutral ACK may relinquish
        // this MCP's writer claim. A departed MC03 must not hold MC02,
        // MC04 or MC05 hostage after their independent neutral ACKs.
        const release=await releaseAcknowledgedAiLeases(
          neutral.acknowledged,leaseEpoch);
        released=release.released;
        // Only independently acknowledged lease releases clear this
        // MCP's hold; a dead child does not block healthy peers.
        if(release.unconfirmed.length)markAiUncertain(release.unconfirmed);
        clearReconciledAiSlots(released);
        if(neutral.unconfirmed.length)markAiUncertain(neutral.unconfirmed);
        if(neutral.unconfirmed.length)throw new Error('AI_RELEASE_UNCONFIRMED');
        if(release.unconfirmed.length) {
          throw new Error('AI_WRITER_LEASE_RELEASE_UNCONFIRMED');
        }
        return {schema:1,slots,status:'NATIVE_NEUTRAL_WRITES_ACKNOWLEDGED',
          writerLease:'NATIVE_RELEASED_AFTER_NEUTRAL_ACK',
          actualGameKeyRelease:'NOT_VERIFIED',gameplayObserved:false};
      } catch(error) {
        if(leaseAttempted && (leaseHeld||!definitelyDeniedWriterLease(error))) {
          // Partial ACKs that were ALSO released stay reconciled. A
          // malformed/uncertain group release or missing child cannot
          // unlock any other possibly held source channel.
          markAiUncertain(slots.filter(slot=>!released.includes(slot)));
        }
        throw error;
      } finally {inFlight=false;}
    }
    if (name==='evaluate_experimental_multi_client_outcomes') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['actions','receipt','after','reviews']);
      // The same compiler defines which AI slots and outcome categories
      // were requested. A caller cannot invent an evaluation of MC01
      // human actions or an unrelated attack/interaction.
      const scheduled=prepareMultiClientPulseBatches(config.mode,args.actions);
      if(args.receipt?.actionFingerprint!==fingerprintPulsePlan(config.mode,scheduled)) {
        throw new Error('TEAM_ACTION_RECEIPT_MISMATCH');
      }
      if(multiSequence!==null||inFlight) {
        throw new Error('Native operation in progress; do not overlap requests');
      }
      if(!Array.isArray(args.receipt?.visualBaseline)) {
        throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
      }
      const reading=scheduled.slots.map(slot=>visualObserver.read(slot,{
        includeImage:false,
      }).metadata);
      // This read does NOT generate a screenshot. The caller-supplied
      // before/after images must have been viewed separately. This only
      // binds recent cache identities and timestamps to an unverified
      // model visual report; it cannot certify Minecraft gameplay.
      const comparison=summarizeTeamVisualState(
        config.mode,scheduled.slots,reading,args.receipt.visualBaseline);
      return evaluateTeamActionFeedback(
        config.mode,args.actions,args.receipt,comparison,
        args.after,args.reviews??[]);
    }
    if (name==='run_experimental_observed_actions') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['actions','basis']);
      const schedule=prepareMultiClientPulseBatches(config.mode,args.actions);
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      // Compile and validate the caller's exact visual baseline even if
      // reconnect recovery is still required. The existing observer must
      // have delivered the SAME current PNG for every selected AI slot;
      // metadata-only samples and malformed identities never authorize
      // input or imply the model actually reviewed the pixels.
      let state=verifyTeamObservationBasis(
        config.mode,schedule.slots,args.basis,
        schedule.slots.map(slot=>visualObserver.read(slot,{
          includeImage:false,
        }).metadata));
      refuseUncertain(schedule.slots);
      // A frame was verified when CAPTURED, not at action time. A client can
      // exit/restart between cache collection and AI dispatch. Revalidate
      // every exact PID/birth/session/HWND through the ORIGINAL native CLI
      // before acquiring any active writer lease. Keep this existing MCP
      // mutation gate during the bounded read-only checks; no concurrent
      // action can pass through this handler while they are in progress.
      inFlight=true;
      try {
        for(const frame of state.frames) {
          let live;
          try {
            live=await readNative('target',frame.slot);
          } catch {
            // Unavailable native identity is NOT permission to use cached
            // imagery for a new input batch. Do not guess a replacement PID.
            throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
          }
          // Native target inspection may complete after MCP stdin closed.
          // No following slot check or lease may be started in that case.
          requireOpen();
          if(!sameNativeVisualTarget(live,frame.baseline?.captureIdentity)) {
            // Known different native child: revoke only this slot's obsolete
            // visual cache and GPU producer. Do not touch another game.
            visualObserver.stop([frame.slot]);
            gpuSource?.stop([frame.slot]);
            throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
          }
        }
        // A sampler may finish, fail, or advance a frame while awaiting the
        // native checks. Recheck the exact original per-slot baseline after
        // ALL awaits, before the single controller is allowed to dispatch.
        state=verifyTeamObservationBasis(
          config.mode,schedule.slots,args.basis,
          schedule.slots.map(slot=>visualObserver.read(slot,{
            includeImage:false,
          }).metadata));
        refuseUncertain(schedule.slots);
        requireOpen();
      } finally {
        inFlight=false;
      }
      // There is no await between releasing the read-only preflight gate
      // and entering the existing exclusive native action path below.
      // Native lease acquisition still validates its exact original child;
      // this preflight is not atomic Windows/gameplay attestation.
      // Acquire can await the original actor while the delivered images
      // age out or advance. Carry the same exact source-visible baseline
      // into the active native path, rather than trusting preflight alone.
      const observedRound={slots:[...schedule.slots],
        basis:state.frames.map(frame=>frame.baseline),
        dispatchStartedAtMs:null};
      observedSequence=observedRound;
      try {
        const receipt=await callTool('run_experimental_multi_client_actions',{
          actions:args.actions,
        });
        // Do not mislabel pre-lease waiting as action transport time.
        if(!Number.isSafeInteger(observedRound.dispatchStartedAtMs)) {
          throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
        }
        const transportStartedAt=new Date(
          observedRound.dispatchStartedAtMs).toISOString();
        const transportCompletedAt=new Date().toISOString();
        const prior=observedRound.basis;
        return {...receipt,mode:config.mode,visualBaseline:prior,
          actionFingerprint:fingerprintPulsePlan(config.mode,schedule),
          transportStartedAt,transportCompletedAt,
          decision:'EXTERNAL_AI_DECISION_NOT_VERIFIED_BY_MCP',
          actionResult:'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE',
          nextObservation:{
            tool:'get_multi_client_visual_state',
            arguments:{slots:schedule.slots,since:prior},
          },
          nextEvaluation:{tool:'evaluate_experimental_multi_client_outcomes',
            requiredInputs:['actions','receipt','after','reviews']},
          note:'Compare newer per-client samples and explicitly review PNGs via read_visual_observation; unchanged or changed PNG bytes alone cannot prove Minecraft movement.'};
      } finally {
        if(observedSequence===observedRound) observedSequence=null;
      }
    }
    if (name==='run_experimental_multi_client_actions') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['actions']);
      // Compile the FULL cross-slot plan before any input request.
      const schedule=prepareMultiClientPulseBatches(config.mode,args.actions);
      refuseUncertain(schedule.slots);
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      // One completely new epoch per authored cross-client action.
      // No native lease may adopt stale frames from a prior invocation.
      const leaseEpoch=makeAiLeaseEpoch();
      const abort=new AbortController();
      const run={abort,slots:schedule.slots};
      multiSequence=run;
      let dispatched=false,transportConfirmed=false;
      let nativeNeutralReceipt=null;
      let leaseAttempted=false,leaseHeld=false;
      let leaseDefinitelyDenied=false;
      let neutralUnconfirmed=[],released=[];
      try {
        leaseAttempted=true;
        // All-or-nothing native cohort claim. A definitely denied foreign
        // writer never touched an input and needs no added recovery marker.
        try {
          await changeAiLease('acquire',schedule.slots,leaseEpoch,false);
        } catch(error) {
          leaseDefinitelyDenied=definitelyDeniedWriterLease(error);
          throw error;
        }
        leaseHeld=true;
        // A previously awaited native acquire may return after shutdown.
        // Refuse new active input; finally still releases the confirmed
        // neutral lease via the same original controller.
        if(closed||abort.signal.aborted)throw new Error('AI_SEQUENCE_ABORTED');
        // A visual source could disappear during the await. Do not
        // acquire a lease and then send the already-invalidated action.
        refuseUncertain(schedule.slots);
        if(observedSequence!==null) {
          // Reuse the existing observer metadata; never take another
          // screenshot, start a worker or add a controller. Fail closed
          // when even one original model-reviewed image has advanced,
          // expired, changed identity, or lost its delivery proof.
          const round=observedSequence;
          if(round.slots.length!==schedule.slots.length ||
             round.slots.some((slot,i)=>slot!==schedule.slots[i])) {
            throw new Error('TEAM_OBSERVATION_STALE_OR_REPLACED');
          }
          verifyTeamObservationBasis(config.mode,schedule.slots,round.basis,
            schedule.slots.map(slot=>visualObserver.read(slot,{
              includeImage:false,
            }).metadata));
          requireOpen();
          if(abort.signal.aborted)throw new Error('AI_SEQUENCE_ABORTED');
          refuseUncertain(schedule.slots);
          round.dispatchStartedAtMs=Date.now();
        }
        // No await between the final source check and native batch request.
        dispatched=true;
        const receipt=await aiMultiBatch(config.executable,schedule.batches,
          {signal:abort.signal,writerId,mode:config.mode,leaseEpoch});
        if(receipt?.schema!==1||
           JSON.stringify(receipt?.slots)!==JSON.stringify(schedule.slots)||
           receipt?.status!=='EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED'||
           receipt?.gameplayObserved!==false) {
          throw new Error('AI_INPUT_NATIVE_RESULT_UNVERIFIED');
        }
        if(abort.signal.aborted)throw new Error('AI_SEQUENCE_ABORTED');
        transportConfirmed=true;
        nativeNeutralReceipt=receipt;
        return receipt;
      }finally{
        // A failed/aborted action may already have published packets.
        // Keep the original ENTIRE cohort quarantined until the model
        // explicitly requests recovery, even when neutral ACKs arrive.
        if(dispatched&&!transportConfirmed)markAiUncertain(schedule.slots);
        if(leaseHeld) {
          // A claim that never dispatched input remains initially neutral.
          // The original CLI has ALREADY sent independent neutral to
          // every touched slot before emitting its exact-content ACK.
          // Skip 4/5 duplicate CLI process/IPC calls only for the
          // canonical sender + fully confirmed exact-slot neutral proof.
          // Aborted/uncertain/fake transport responses still take the
          // existing conservative independent neutral/quarantine path.
          const nativeNeutralConfirmed=canReuseVerifiedNativeBatchNeutral(
            nativeNeutralReceipt,schedule.slots,{
              verifiedSender:aiMultiBatch===sendExperimentalMultiClientBatchNative,
              transportConfirmed,
            });
          const neutral=!dispatched || nativeNeutralConfirmed?
            {acknowledged:[...schedule.slots],unconfirmed:[]}:
            await releaseAiSlots(schedule.slots,leaseEpoch);
          neutralUnconfirmed=neutral.unconfirmed;
          if(neutralUnconfirmed.length)markAiUncertain(neutralUnconfirmed);
          // Each selected native child can disappear between its
          // neutral ACK and the final release. Do not send a grouped
          // release which lets one departed slot strand all peers.
          const release=await releaseAcknowledgedAiLeases(
            neutral.acknowledged,leaseEpoch);
          released=release.released;
          if(release.unconfirmed.length)markAiUncertain(release.unconfirmed);
        }
        if(leaseAttempted&&!leaseHeld&&!leaseDefinitelyDenied) {
          // Native acquire ACK was ambiguous. Never assume release or
          // replay; each native writer claim expires without a daemon.
          markAiUncertain(schedule.slots);
        }
        if(multiSequence===run)multiSequence=null;
        inFlight=false;
        if(neutralUnconfirmed.length)throw new Error('AI_RELEASE_UNCONFIRMED');
        if(leaseHeld&&released.length!==schedule.slots.length) {
          throw new Error('AI_WRITER_LEASE_RELEASE_UNCONFIRMED');
        }
        // A Stop/shutdown may arrive AFTER the original batch ACK while
        // individual native lease releases still await their CLI replies.
        // A prior successful transport is not a completed MCP transaction
        // once the operator has cancelled during this final cleanup.
        // Preserve stronger unconfirmed neutral/lease errors above.
        if(transportConfirmed&&(closed||abort.signal.aborted)) {
          markAiUncertain(schedule.slots);
          throw new Error('AI_SEQUENCE_ABORTED');
        }
      }
    }
    if (name==='stop_experimental_mc02_actions') {
      if(!mc02Enabled)throw new Error('MC02_EXPERIMENT_DISABLED');
      keysExactly(args,['slot']);
      if(args.slot!==2)throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
      if(mc02SequenceAbort!==null) {
        mc02SequenceAbort.abort();
        return {schema:1,slot:2,status:'CANCELLATION_REQUESTED',
          neutralRelease:'PENDING_SEQUENCE_COMPLETION_OR_NATIVE_TTL',
          gameplayObserved:false};
      }
      // An earlier one-shot pulse can still be held without an active
      // sequence. Ask the exact native owner to clear MC02 input, never
      // the global keyboard or MC01. This write is also allowed when the
      // human foreground gate has already closed.
      if(inFlight)throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        const receipt=await mc02Input(config.executable,{
          keys:0,dx:0,dy:0,ttlMs:1,requestedAtMs:Date.now(),
        });
        if(receipt?.schema!==1||receipt?.slot!==2||
           receipt?.gameplayObserved!==false||
           receipt?.status!=='NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED') {
          throw new Error('MC02_INPUT_NATIVE_RESULT_UNVERIFIED');
        }
        return {schema:1,slot:2,
          status:'MC02_NATIVE_NEUTRAL_STATE_WRITE_ACKNOWLEDGED',
          actualGameKeyRelease:'NOT_VERIFIED',
          gameplayObserved:false,
          note:'State cleared by retained MC02 channel; no proof that Minecraft read the release.'};
      } finally {inFlight=false;}
    }
    if (name==='run_experimental_mc02_actions') {
      if(!mc02Enabled)throw new Error('MC02_EXPERIMENT_DISABLED');
      keysExactly(args,['slot','actions']);
      if(args.slot!==2)throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
      // Pure compile before native admission: malformed timelines cannot
      // spawn a child process or send a single MC02 game-input packet.
      const batch=prepareMc02PulseBatch(args.actions);
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      const abort=new AbortController();
      mc02SequenceAbort=abort;
      let dispatched=false,nativeNeutralConfirmed=false;
      let cleanupObsolete=false,neutralConfirmed=false;
      try {
        dispatched=true;
        const result=await mc02Batch(config.executable,batch.pulses,
          {signal:abort.signal});
        if(result?.schema!==1||result?.slot!==2||
           result?.status!=='EXPERIMENTAL_MC02_SEQUENCE_TRANSPORT_ACKNOWLEDGED'||
           result?.pulsesAcknowledged!==batch.count||
           result?.gameplayObserved!==false) {
          throw new Error('MC02_INPUT_NATIVE_RESULT_UNVERIFIED');
        }
        if(abort.signal.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
        nativeNeutralConfirmed=canReuseVerifiedMc02BatchNeutral(
          result,batch.count,{
            verifiedSender:mc02Batch===sendExperimentalMc02BatchNative,
            transportConfirmed:true,
          });
        return result;
      } catch(error) {
        // Native proof of an obsolete, NEVER-published cleanup cannot
        // be replaced by a later fresh neutral without canceling an
        // unrelated writer. This is an error, not a success or key-up ACK.
        cleanupObsolete=confirmedObsoleteMc02BatchCleanup(error,{
          verifiedSender:mc02Batch===sendExperimentalMc02BatchNative,
        });
        throw error;
      } finally {
        // Confirmed canonical success already neutralized MC02; proven
        // obsolete cleanup belongs to an earlier timeline. Otherwise
        // retain independent exact-channel neutral for uncertain/killed,
        // malformed or injected senders.
        if(dispatched&&!nativeNeutralConfirmed&&!cleanupObsolete) {
          try {
            const receipt=await mc02Input(config.executable,{
              keys:0,dx:0,dy:0,ttlMs:1,requestedAtMs:Date.now(),
            });
            neutralConfirmed=receipt?.schema===1&&receipt?.slot===2&&
              receipt?.gameplayObserved===false&&
              receipt?.status==='NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED';
          } catch {neutralConfirmed=false;}
        }
        if(mc02SequenceAbort===abort)mc02SequenceAbort=null;
        inFlight=false;
        if(dispatched&&!nativeNeutralConfirmed&&!neutralConfirmed&&!cleanupObsolete)
          throw new Error('MC02_RELEASE_UNCONFIRMED');
        // A proven obsolete cleanup is still a rejected action, never
        // a successful native batch or gameplay key-up observation.
        // A late Stop also cannot be converted into earlier success.
        if(abort.signal.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      }
    }
    if (name==='send_experimental_mc02_input') {
      if(!mc02Enabled) throw new Error('MC02_EXPERIMENT_DISABLED');
      const requestedAtMs=Date.now(); // before two native target inspections
      keysExactly(args,['slot','buttons','lookDX','lookDY','ttlMs']);
      if(args.slot!==2) throw new Error('MC02_INPUT_INVALID_COMMAND');
      const keys=mc02ButtonMask(args.buttons);
      const dx=args.lookDX??0,dy=args.lookDY??0,ttlMs=args.ttlMs??150;
      if(!Number.isSafeInteger(dx)||dx< -127||dx>127||
         !Number.isSafeInteger(dy)||dy< -127||dy>127||
         !Number.isSafeInteger(ttlMs)||ttlMs<1||ttlMs>500) {
        throw new Error('MC02_INPUT_INVALID_COMMAND');
      }
      if(exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        // The retained Rust session controller already owns and pins
        // BOTH exact Minecraft game processes and their root HWNDs, and
        // verifies MC01 foreground before the shared-memory write.
        // Do not start two extra read-only target CLI processes here:
        // those synchronous preflights consume most or all of the 500ms
        // command deadline without adding authoritative protection.
        // No fallback to global OS input or a foreign game process.
        const receipt=await mc02Input(config.executable,
          {keys,dx,dy,ttlMs,requestedAtMs});
        if(receipt?.schema!==1||receipt?.slot!==2||
           receipt?.gameplayObserved!==false||
           receipt?.status!=='NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED') {
          throw new Error('MC02_INPUT_NATIVE_RESULT_UNVERIFIED');
        }
        return {...receipt,ttlMs,
          inputEffect:'UNVERIFIED_IN_MINECRAFT_EDUCATION'};
      } finally {inFlight=false;}
    }
    if (name==='assess_input_route') {
      keysExactly(args,['actions']);
      // Validate all actions and the human/AI ownership before touching
      // the native controller. The request only inspects existing windows.
      const required=requiredInputSlots(config.mode,args.actions);
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      const started=Date.now();
      try {
        const observed=[];
        for (const slot of required.observedSlots) {
          observed.push(await readNative('target',slot));
        }
        const elapsed=Date.now()-started;
        return {
          ...assessInputRoute(config.mode,args.actions,observed,{
            observedAtMs:Date.now(),elapsedMs:elapsed,
          }),
          transactionEngine:inputExecutionStatus(),
        };
      } finally {inFlight=false;}
    }
    if (name==='plan_peer_session') {
      keysExactly(args,[]);
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        const report=await readNative('inspect');
        return planPeerSession(config.mode,report);
      } finally {inFlight=false;}
    }
    if (name==='agent_capabilities') {
      keysExactly(args,['publishToDesktop','inspectProfiles']);
      if((args.publishToDesktop!==undefined && typeof args.publishToDesktop!=='boolean')||
         (args.inspectProfiles!==undefined && typeof args.inspectProfiles!=='boolean')||
         (args.inspectProfiles===true&&args.publishToDesktop===true)) {
        throw new Error('Invalid tool arguments');
      }
      // Reuse the existing Inspect admission, fresh native receipt and
      // public projection. No native call for the ordinary capability read.
      const profileInspection=args.inspectProfiles===true?
        await callTool('inspect_profiles',{}):null;
      requireOpen();
      // One bounded read of already-owned MCP memory. It cannot attest
      // external desktop control, successful input, an authenticated QA world,
      // or whether a separately run Skills Probe command was actually sent.
      const activity=projectAgentActivity({
        mode:config.mode,atMs:qaClock(),
        multiEnabled,legacyEnabled:mc02Enabled,
        multiSlots:multiSequence?.slots??[],
        legacyPending:mc02SequenceAbort!==null,
        uncertainSlots:multiEnabled?[...uncertainAiSlots]:[],
        startupReconcileSlots:multiEnabled?[...startupReconcileSlots]:[],
        probePlans:[...probeQaPlans.entries()].map(([slot,plan])=>({
          slot,preparedAt:plan.preparedAt,
          terminalStatus:plan.terminalReport?.status??null,
          terminalCounts:plan.terminalReport?.counts??null,
          expectedCount:plan.terminalReport?.expectedCount??null,
        })),
        probeBusy:[...probeQaBusy],
        visualSamplers:typeof visualObserver.activity==='function'?
          visualObserver.activity():[],
      });
      const result={...capabilities(config.mode,mc02Enabled,multiEnabled),activity,
        ...(profileInspection!==null?{profileInspection}:{}),
        // Configuration is not provider liveness, runtime proof, or approval.
        configuredSources:{
          native:!!config.executable,
          visual:gpuSource?'WGC':config.cuaExecutable?'CUA':'NOT_CONFIGURED',
          semanticUi:!!config.cuaExecutable,
          evidence:!!config.evidenceDirectory,
          diagnostics:!!config.diagnosticsDirectory,
          probe:!!config.probeZip,
          mapContract:!!config.mapContractPath,
          testingArchive:!!config.testingArchiveDirectory,
        }};
      if(args.publishToDesktop!==true)return result;
      try {
        const delivery=await publishActivity(config.executable,activity);
        if(delivery?.status!=='REPORTED_TO_EXISTING_NATIVE_CONTROLLER')
          throw new Error('Status delivery not acknowledged');
        return {...result,desktopDelivery:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER'};
      } catch {
        return {...result,desktopDelivery:'STATUS_DELIVERY_UNAVAILABLE'};
      }
    }
    if (name==='read_game_evidence') {
      keysExactly(args,['slot']);
      const slot=slotOf(args.slot);
      // Preserve the public tool name and slot/status/events layout, but
      // never forward arbitrary debugger JSON fields to the model. Reuse
      // the existing allowlisted, bounded summary event projection.
      const summarized=summarizeEvidenceReports([slot],[
        await evidence(config.evidenceDirectory,slot),
      ]);
      const source=summarized.sources[0];
      return {
        schema:1,slot,status:source.status,
        events:summarized.timeline,records:summarized.records,
        truncated:source.truncated,
        incompleteTrailingLineIgnored:source.incompleteTrailingLineIgnored,
        attribution:'OPERATOR_NAMED_UNVERIFIED',
        response:'BOUNDED_PROJECTION_NO_RAW_SKILLS_PROBE_JSON',
        verdict:summarized.verdict,
        note:'Events contain bounded unverified metadata only, not raw captured payloads or gameplay acceptance.',
      };
    }
    if (name==='inspect_profiles') {
      keysExactly(args,[]);
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        return await readNative('inspect');
      } finally {inFlight=false;}
    }
    if (name==='launch_profile') {
      keysExactly(args,['slot']);const slot=slotOf(args.slot);
      if (config.mode==='human-plus-four-ai' && slot===1) throw new Error('MC01 is reserved for the human host in hybrid mode');
      if (exclusiveBusy()) throw new Error('Native operation in progress; do not overlap requests');
      inFlight=true;
      try {
        // An already-running native controller never inherits the MCP
        // process environment. Carry authorization as an authenticated
        // launch request, or the MC02 input reader would not exist.
        const operation=multiEnabled?
          (config.mode==='five-ai'?'launch-ai-five':'launch-ai-hybrid') :
          (mc02Enabled&&slot===2?'launch-mc02-input':'launch');
        const report=await readNative(operation,slot);
        // A successful *new* native experimental launch creates a fresh
        // exact-process channel with neutral initial input state. It
        // cannot adopt an already-running game. Only a non-attention
        // original child reported STARTING/RUNNING clears that slot's
        // startup hold; an Inspect or older running window never does.
        if(multiEnabled && ['STARTING','RUNNING'].includes(
              report.profiles[slot-1].state) &&
           report.profiles[slot-1].needsAttention===false) {
          uncertainAiSlots.delete(slot);
          startupReconcileSlots.delete(slot);
        }
        return report;
      } finally {inFlight=false;}
    }
    throw new Error('Unknown tool');
  }
  return {callTool,listTools:()=>toolsForMode(mc02Enabled,multiEnabled),isClosed:()=>closed,shutdown:()=>{
    if(closed)return;
    closed=true;
    evidenceBaselines.clear();
    // Native input may already have published packets. Quarantine the
    // touched cohort and retain the existing exact-owner finally cleanup.
    if(multiSequence)markAiUncertain(multiSequence.slots);
    mc02SequenceAbort?.abort();
    multiSequence?.abort.abort();
    if (gpuStartup) gpuStartup.cancelled=true;
    visualObserver.shutdown();
    gpuSource?.shutdown();
  }};
}

