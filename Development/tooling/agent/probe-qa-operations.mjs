// One source-only Probe QA checkpoint owner per existing operation core.
// Native run correlation, evidence parsing, and approved-source checks retain their original owners.
import {createHash,randomBytes} from 'node:crypto';
import {slotOf,keysExactly} from './agent-contract.mjs';
import {EVIDENCE_LIMIT} from './evidence-reader.mjs';
import {testingRecoveryNextAction} from './native-transport.mjs';
import {reviewProbeRun,compactProbeQaResult,probeQaProofGate} from './probe-qa.mjs';
import {planMapTests,compactMapTestPlan,composeMapQaReport} from './map-test-plan.mjs';

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

export function createProbeQaOperations(config,{
  evidence,coordinateTesting,inspectTesting,archiveTesting,readArchivedTesting,
  findArchivedTestingSlot,reviewTestingArchive,probeCatalog,probeRevisionStatus,
  mapContract,mapRevisionStatus,qaClock,requireOpen,exclusiveBusy,callTool,
}) {
  // One volatile plan per slot; not a Minecraft world manager or an auto-run loop.
  const probeQaPlans=new Map();
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
  // One private Probe checkpoint owner; probe_qa is the only public ingress.
  async function prepareProbeQa(slot) {
    requireOpen();
    if(config.mode==='human-plus-four-ai'&&slot===1) {
      throw new Error('MC01 human host is protected from AI computer-use targeting');
    }
    // Preparing again invalidates any older result token for the same slot.
    // A new local checkpoint cannot inherit a previous native run ID.
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
       /^[a-f0-9]{64}$/.test(captureFileIdentity)) {
      for(const [otherSlot,other] of probeQaPlans) {
        if(otherSlot!==slot&&other.captureCursor?.fileIdentity===captureFileIdentity) {
          return {schema:1,slot,status:'CAPTURE_SLOT_COLLISION',
            commandSent:false,gameplayAcceptance:'NOT_VERIFIED'};
        }
      }
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
      next:'After an authorized operator or verified AI Computer Use executes the original command, call probe_qa for this same slot.',
      commandSent:false,gameplayAcceptance:'NOT_VERIFIED'};
  }
  async function readProbeQaResult(slot,sessionId) {
    requireOpen();
    const session=probeQaPlans.get(slot);
    if(!session||session.sessionId!==sessionId) {
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
  // Only the existing core admits external calls. Nested target/capability
  // operations deliberately return through its same admission path.
  async function callProbeTool(args) {
    if(args && Object.hasOwn(args,'inspectRun')) {
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
    if(args && Object.hasOwn(args,'archivedRunId')) {
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
    // Calls without the flags use the same private checkpoint.
    // Desktop delivery is only a short-lived presentation mirror, never
    // another result store, runner, approval or proof of gameplay input.
    if(args &&
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
            // The fresh owner-matching reservation can already be FINAL.
            // Do not send a second finalize for that exact immutable report.
            // A different digest/outcome is a conflict, never a repair/replay.
            terminalReference=runRef.phase==='FINAL'?runRef:
              await coordinateTesting(config.executable,{
                operation:'finalize',slot,runId,ownerId:testingOwnerId,
                reportedStatus:outcome,reportDigest,
              });
            if(terminalReference?.phase!=='FINAL'||
               terminalReference.slot!==slot||terminalReference.runId!==runId||
               terminalReference.ownerMatch!==true||
               terminalReference.reportedStatus!==outcome||
               terminalReference.reportDigest!==reportDigest) {
              throw new Error('TESTING_NATIVE_RECEIPT_INVALID');
            }
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
      // One user-facing tool; the caller never carries a private session ID.
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
        try {prepared=await prepareProbeQa(slot);}
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
        raw=await readProbeQaResult(slot,plan.sessionId);
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

  function activitySnapshot() {
    return {
      probePlans:Array.from(probeQaPlans,([slot,plan])=>({
        slot,preparedAt:plan.preparedAt,
        terminalStatus:plan.terminalReport?.status??null,
        terminalCounts:plan.terminalReport?.counts??null,
        expectedCount:plan.terminalReport?.expectedCount??null,
      })),
      probeBusy:[...probeQaBusy],
    };
  }

  return {callProbeTool,activitySnapshot};
}
