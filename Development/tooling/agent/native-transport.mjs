// Same-image CLI transport and exact receipt validation; no session owner.
import {execFile} from 'node:child_process';
import {createHash,randomBytes} from 'node:crypto';
import {performance} from 'node:perf_hooks';
import path from 'node:path';
import {promisify} from 'node:util';
import {GAME_BUTTON_BITS} from './input-timeline.mjs';

const execFileAsync=promisify(execFile);
const VALID_STATES = new Set(['STOPPED','STARTING','RUNNING','NOT_RESPONDING','ATTENTION']);

export function validateNativeWindowTarget(value,slot) {
  if (!value || value.schema!==1 || value.ok!==true ||
      value.operation!=='target' || value.slot!==slot ||
      !Number.isSafeInteger(value.pid) || value.pid<=0 ||
      !Number.isSafeInteger(value.windowId) || value.windowId<=0 ||
      typeof value.createdFileTime!=='string' ||
      !/^\d{1,20}$/.test(value.createdFileTime) ||
      !Number.isSafeInteger(value.sessionId) || value.sessionId<0 ||
      typeof value.foreground!=='boolean' ||
      value.visible!==true || value.minimized!==false ||
      value.verification!=='SAME_SESSION_PID_BIRTH_ROOT_HWND' ||
      value.gameplayInput!=='NOT_VERIFIED') {
    throw new Error('Native Minecraft window target is not verified');
  }
  return value;
}

// Observation cache identity is evidence, never an input target authority.
// A batch grounded in old pixels must verify the SAME exact native process
// immediately before requesting new writer leases. Rust remains the sole
// final owner and rechecks its original child on every native input frame.
export function sameNativeVisualTarget(target,frameIdentity) {
  return !!target && !!frameIdentity &&
    target.pid===frameIdentity.pid &&
    target.windowId===frameIdentity.windowId &&
    target.sessionId===frameIdentity.sessionId &&
    target.createdFileTime===frameIdentity.createdFileTime;
}

export function computerUseTarget(value,slot,mode) {
  validateNativeWindowTarget(value,slot);
  return {
    schema:1,slot,mode,role:'AI',
    target:{kind:'window',pid:value.pid,window_id:value.windowId},
    identity:{pid:value.pid,windowId:value.windowId,
      createdFileTime:value.createdFileTime,sessionId:value.sessionId},
    isForeground:value.foreground,
    verification:'NATIVE_PROCESS_AND_WINDOW_VERIFIED_AT_READ_TIME',
    suggestedExternalTool:'cua-driver get_window_state',
    capture:'NOT_CAPTURED_BY_LAZY_TESTER',
    gameInput:'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED',
    noFocusSteal:true,
    note:'Window identity expires when the game/window changes. Revalidate before and after external computer-use actions. Never escalate to foreground in hybrid mode.',
  };
}

export function validateNativeReport(value, operation) {
  if (!value || value.schema!==1 || value.ok!==true || value.operation!==operation ||
      !Array.isArray(value.profiles) || value.profiles.length!==5) {
    throw new Error('Native result unavailable or inconsistent');
  }
  // The Rust agent_cli already emits a minimal five-profile view. Rebuild
  // that exact view here, instead of returning its original JSON object:
  // future native fields (paths, account info or issue text) must not become
  // publicly visible just because the five mandatory fields were valid.
  const profiles=value.profiles.map((p,i)=>{
    if (!p || p.slot!==i+1 || !VALID_STATES.has(p.state) ||
        typeof p.responding!=='boolean' || typeof p.hasWindow!=='boolean' ||
        typeof p.needsAttention!=='boolean') {
      throw new Error('Invalid profile state from native controller');
    }
    return {
      slot:p.slot,state:p.state,hasWindow:p.hasWindow,
      responding:p.responding,needsAttention:p.needsAttention,
    };
  });
  return {
    schema:1,ok:true,operation,profiles,
    evidence:'NATIVE_SESSION_ONLY_NOT_GAMEPLAY_OR_ACCOUNT_SIGNIN',
  };
}
// Read-only native observation of the original controller's in-memory
// Testing reservation. Owner token equality is not proof that a particular
// AI process is alive/dead; no operator approval/transfer exists here.
// Validate the exact read-only source fields independently of CLI transport.
// Strict parser prevents accidental claims of verified owner death/approval.
export function parseTestingInspectionReceipt(message,slot) {
  const obs=message?.inspection;
  const keys=['schema','slot','phase','runId','ownerTokenMatch',
    'reportedStatus','reportDigest','ownerProcessProof',
    'operatorApproval','transferAllowed','recoveryAction','provenance'];
  const fail=()=>{throw new Error('TESTING_NATIVE_INSPECTION_RECEIPT_INVALID');};
  if(message?.schema!==1||message.ok!==true||
     message.operation!=='testing-inspect'||
     !obs||typeof obs!=='object'||Array.isArray(obs)||
     Object.keys(obs).length!==keys.length||
     keys.some(key=>!Object.hasOwn(obs,key))||
     obs.schema!==1||obs.slot!==slot||
     !['NO_RECORD','RESERVED','FINAL'].includes(obs.phase)||
     typeof obs.ownerTokenMatch!=='boolean'||
     !['NOT_BOUND_TO_PROCESS_HANDLE',
       'BOUND_PARENT_RUNNING_AT_INSPECTION',
       'BOUND_PARENT_EXIT_CONFIRMED',
       'BOUND_PARENT_STATUS_UNKNOWN'].includes(obs.ownerProcessProof)||
     obs.operatorApproval!=='NOT_AVAILABLE_IN_PROTOCOL'||
     obs.transferAllowed!==false||
     obs.provenance!=='RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF')
     fail();
  const validId=value=>typeof value==='string'&&/^[a-f0-9]{32}$/.test(value);
  const validDigest=value=>typeof value==='string'&&/^[a-f0-9]{64}$/.test(value);
  const outcomes=['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW','INCOMPLETE','BLOCKED'];
  if(obs.phase==='NO_RECORD') {
    if(obs.runId!==null||obs.ownerTokenMatch!==false||
       obs.ownerProcessProof!=='NOT_BOUND_TO_PROCESS_HANDLE'||
       obs.reportedStatus!==null||obs.reportDigest!==null||
       obs.recoveryAction!=='CHECK_ARCHIVE_AND_CONTROLLER_EPOCH')fail();
  }else if(obs.phase==='RESERVED') {
    if(!validId(obs.runId)||obs.reportedStatus!==null||
       obs.reportDigest!==null||
       obs.recoveryAction!=='REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL')
       fail();
  }else {
    if(!validId(obs.runId)||!outcomes.includes(obs.reportedStatus)||
       !validDigest(obs.reportDigest)||
       obs.recoveryAction!=='READ_FINAL_REPORT_NO_OWNER_TRANSFER')fail();
  }
  return obs;
}

// Read-only operator guidance. NEVER treats a missing archive, parent exit,
// token match or checksum as authorization for another Testing owner.
export function testingRecoveryNextAction(inspection,archiveReview) {
  if(archiveReview?.status==='INTEGRITY_UNAVAILABLE_OR_CONFLICT'||
     archiveReview?.status==='CONFLICT_RESERVED_HAS_FINAL_ARCHIVE'||
     archiveReview?.status==='CONFLICT_FINAL_REFERENCE_MISMATCH')
    return 'REVIEW_TESTING_ARCHIVE_CONFLICT_NO_TRANSFER';
  if(inspection.phase==='FINAL')return 'READ_FINAL_REPORT_NO_OWNER_TRANSFER';
  if(inspection.phase==='NO_RECORD')
    return archiveReview?.status==='HISTORICAL_SUMMARY_PRESENT'?
      'READ_HISTORICAL_RESULT_NO_AUTOMATIC_RESTART':
      'CHECK_ARCHIVE_AND_CONTROLLER_EPOCH';
  if(inspection.ownerProcessProof==='BOUND_PARENT_RUNNING_AT_INSPECTION')
    return 'WAIT_FOR_LIVE_TESTING_OWNER_NO_TRANSFER';
  if(inspection.ownerProcessProof==='BOUND_PARENT_EXIT_CONFIRMED')
    return archiveReview?.status==='NO_COMMITTED_SUMMARY_FOUND'?
      'AWAIT_SEPARATE_OPERATOR_APPROVAL_NO_TRANSFER_AVAILABLE':
      'RECONCILE_TESTING_ARCHIVE_BEFORE_ANY_TRANSFER';
  return 'REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL';
}

export async function inspectTestingRunNative(executable,{slot,ownerId}) {
  if(!executable)throw new Error('Native Testing controller not configured');
  if(!Number.isInteger(slot)||slot<1||slot>5||
     !/^[a-f0-9]{32}$/.test(ownerId??''))
    throw new Error('Invalid read-only Testing inspection');
  let raw;
  try {
    raw=await execFileAsync(executable,
      ['--lazy-tester-agent-cli','testing-inspect',String(slot)],{
        env:{...process.env,LAZY_TESTER_TESTING_OWNER_ID:ownerId},
        shell:false,windowsHide:true,timeout:4000,maxBuffer:4096,
      });
  }catch{
    throw new Error('TESTING_NATIVE_INSPECTION_UNAVAILABLE');
  }
  let message;
  try{message=JSON.parse(raw.stdout.trim());}
  catch{throw new Error('TESTING_NATIVE_INSPECTION_RECEIPT_INVALID');}
  return parseTestingInspectionReceipt(message,slot);
}

export async function coordinateTestingRunNative(executable,{
  operation,slot,runId,ownerId,replaceRunId=null,
  reportedStatus=null,reportDigest=null,
}) {
  if(!executable)throw new Error('Native Testing controller not configured');
  if(!['reserve','finalize'].includes(operation)||
    !Number.isInteger(slot)||slot<1||slot>5||
    !/^[a-f0-9]{32}$/.test(runId??'')||
    !/^[a-f0-9]{32}$/.test(ownerId??'')||
    (replaceRunId!==null&&!/^[a-f0-9]{32}$/.test(replaceRunId))) {
    throw new Error('Invalid Testing run correlation request');
  }
  const args=operation==='reserve'?
    ['--lazy-tester-agent-cli','testing-reserve',String(slot),runId,
      replaceRunId??'none']:
    ['--lazy-tester-agent-cli','testing-finalize',String(slot),runId,
      reportedStatus??'',reportDigest??''];
  if(operation==='finalize'&&
      (!['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW','INCOMPLETE','BLOCKED']
        .includes(reportedStatus)||!/^[a-f0-9]{64}$/.test(reportDigest??'')))
    throw new Error('Invalid Testing terminal reference');
  let output;
  try {
    // Same-image native CLI + existing authenticated controller only.
    // Owner correlation is private in env, never argv or UI. Neither it
    // nor a successful native ACK certifies MCP parent/game provenance.
    output=await execFileAsync(executable,args,{
      env:{...process.env,LAZY_TESTER_TESTING_OWNER_ID:ownerId},
      shell:false,windowsHide:true,timeout:4000,maxBuffer:32*1024});
  } catch {
    // An ambiguous native reservation must never fall back to a fresh
    // local checkpoint or start the game through another path.
    throw new Error('TESTING_NATIVE_RESERVATION_UNCONFIRMED');
  }
  let result;
  try {result=JSON.parse(output.stdout.trim());} catch {
    throw new Error('TESTING_NATIVE_RECEIPT_INVALID');
  }
  const run=result?.run;
  if(result?.ok!==true||result.schema!==1||
    result.operation!=='testing-'+operation||
    run?.schema!==1||run.slot!==slot||
    !/^[a-f0-9]{32}$/.test(run.runId??'')||
    !['RESERVED','FINAL'].includes(run.phase)||
    typeof run.ownerMatch!=='boolean'||
    run.provenance!=='NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'||
    !(run.reportedStatus===null||
      ['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW','INCOMPLETE','BLOCKED']
        .includes(run.reportedStatus))||
    !(run.reportDigest===null||/^[a-f0-9]{64}$/.test(run.reportDigest))) {
    throw new Error('TESTING_NATIVE_RECEIPT_INVALID');
  }
  if(run.phase==='RESERVED' &&
     (run.reportedStatus!==null||run.reportDigest!==null)||
     run.phase==='FINAL'&&
     (run.reportedStatus===null||run.reportDigest===null)) {
    throw new Error('TESTING_NATIVE_RECEIPT_INVALID');
  }
  return run;
}

export async function publishAgentActivityNative(executable,report) {
  if(!executable)throw new Error('Native integration is not configured; set --exe');
  if(!report||report.source!=='CURRENT_MCP_PROCESS_MEMORY_ONLY'||
     report.desktopControl!=='NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER'||
     report.minecraftInputEffect!=='NOT_VERIFIED'||
     report.mapGameplayAcceptance!=='NOT_TESTED') {
    throw new Error('Invalid agent activity report');
  }
  const payload=JSON.stringify(report);
  if(Buffer.byteLength(payload,'utf8')>5000)
    throw new Error('Invalid agent activity report');
  // Explicit short same-image CLI status mirror. This is NOT a native
  // input/QA command and never retries an uncertain report delivery.
  let output;
  try {
    output=await execFileAsync(executable,
      ['--lazy-tester-agent-cli','report-agent-activity',payload],
      {shell:false,windowsHide:true,timeout:1500,maxBuffer:4096});
  } catch {
    throw new Error('Local agent status delivery unconfirmed');
  }
  let response;
  try{response=JSON.parse(output.stdout.trim());}catch{
    throw new Error('Local agent status delivery unconfirmed');
  }
  if(response?.schema!==1||response?.ok!==true||
     response.operation!=='report-agent-activity'||
     response.status!=='LOCAL_AGENT_REPORT_RECEIVED_NOT_DESKTOP_CONTROL')
    throw new Error('Local agent status delivery unconfirmed');
  return {status:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER',
    desktopControl:'NOT_VERIFIED'};
}
export async function runNative(executable, operation, slot) {
  if (!executable) throw new Error('Native integration is not configured; set --exe');
  const args = ['--lazy-tester-agent-cli',operation,...(slot === undefined?[]:[String(slot)])];
  let out;
  try {
    // No shell. A launch can start the retained native controller and game; avoid overlapping calls.
    out=await execFileAsync(executable,args,{shell:false,windowsHide:true,timeout:55_000,maxBuffer:64*1024});
  } catch (error) {
    if (error.killed || error.signal) throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    if (typeof error.stdout === 'string') {
      try { const response=JSON.parse(error.stdout.trim());
        if (response?.ok===false && typeof response.code==='string' && /^[A-Z_]+$/.test(response.code)) {
          throw new Error('Native request refused: '+response.code);
        }
      } catch (parsed) { if (parsed.message.startsWith('Native request refused:')) throw parsed; }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  if (out.stdout.length>64*1024) throw new Error('Native result exceeds limit');
  const result=JSON.parse(out.stdout.trim());
  return operation==='target' ?
    validateNativeWindowTarget(result,slot) :
    validateNativeReport(result,operation);
}


// The native input transport is intentionally NOT part of runNative()'s
// inspect/launch/target contract. It is an opt-in one-shot operation on the
// same reviewed lazy-tester.exe and authenticated session controller.
export function mc02ButtonMask(buttons) {
  if(!Array.isArray(buttons)||buttons.length>8||
     new Set(buttons).size!==buttons.length) {
    throw new Error('MC02_INPUT_INVALID_COMMAND');
  }
  let mask=0;
  for(const button of buttons) {
    if(typeof button!=='string' || !Object.hasOwn(GAME_BUTTON_BITS,button)) {
      throw new Error('MC02_INPUT_INVALID_COMMAND');
    }
    mask|=GAME_BUTTON_BITS[button];
  }
  if((mask&3)===3||(mask&12)===12) {
    throw new Error('MC02_INPUT_INVALID_COMMAND');
  }
  return mask;
}
export function sendExperimentalMc02Native(executable,frame) {
  return sendExperimentalAiNative(executable,{...frame,slot:2},{legacy:true});
}

export async function sendExperimentalAiNative(executable,
    {slot,keys,dx,dy,ttlMs,requestedAtMs},
    {legacy=false,writerId,mode,leaseEpoch}={}) {
  const nowMs=Date.now();
  if(!Number.isSafeInteger(requestedAtMs)||
     requestedAtMs>nowMs||nowMs-requestedAtMs>500) {
    throw new Error('MC02_INPUT_REQUEST_STALE');
  }
  if(!executable || !path.isAbsolute(executable) ||
     !Number.isInteger(slot)||slot<1||slot>5||(legacy&&slot!==2) ||
     !Number.isSafeInteger(keys)||keys<0||keys>255 ||
     (keys&3)===3||(keys&12)===12 ||
     !Number.isSafeInteger(dx)||dx< -127||dx>127 ||
     !Number.isSafeInteger(dy)||dy< -127||dy>127 ||
     !Number.isSafeInteger(ttlMs)||ttlMs<1||ttlMs>500) {
    throw new Error('MC02_INPUT_INVALID_COMMAND');
  }
  let stdout;
  try {
    const response=await execFileAsync(executable,[
      '--lazy-tester-agent-cli',legacy?'mc02-input':'ai-input',String(slot),
      String(keys),String(dx),String(dy),String(ttlMs),
      String(requestedAtMs),
    ],{shell:false,windowsHide:true,timeout:6000,maxBuffer:8192,
      ...(legacy?{}:{env:writerEnvironment(writerId,mode,leaseEpoch)})});
    stdout=response.stdout;
  } catch(error) {
    if(error.killed||error.signal) {
      throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    }
    if(typeof error.stdout==='string') {
      try {
        const output=JSON.parse(error.stdout.trim());
        if(output?.ok===false&&/^[A-Z_]{1,40}$/.test(output.code??'')) {
          throw new Error('Native request refused: '+output.code);
        }
      } catch(parsed) {
        if(parsed?.message?.startsWith('Native request refused:')) throw parsed;
      }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  let receipt;
  try {receipt=JSON.parse(stdout.trim());} catch {}
  if(receipt?.schema!==1||receipt?.ok!==true||
     receipt?.operation!==(legacy?'mc02-input':'ai-input')||receipt?.slot!==slot||
     receipt?.delivery!=='PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF'||
     receipt?.gameplayObserved!==false) {
    throw new Error('MC02_INPUT_NATIVE_RESULT_UNVERIFIED');
  }
  return {schema:1,slot,
    status:legacy?'NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED':
      'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
    acknowledgedBy:'RETAINED_NATIVE_CONTROLLER',
    gameplayObserved:false,
    inputBackend:'EXPERIMENTAL_WIN32_POLLING_ONLY',
    note:'A native shared-state write is not proof Minecraft processed movement, camera or attacks.'};
}

// One same-image child for one BOUNDED MC02 action sequence, rather than a
// separate Windows process for each 160ms heartbeat. It never creates a
// second Minecraft controller: each native tick enters existing IPC.
export async function sendExperimentalMc02BatchNative(executable,pulses,{signal}={}) {
  if(!executable||!path.isAbsolute(executable)||!Array.isArray(pulses)||
     pulses.length<1||pulses.length>64) {
    throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  }
  let previous=-1;
  const args=[];
  for(const pulse of pulses) {
    const {atMs,keys,dx,dy,ttlMs}=pulse;
    if(!Number.isSafeInteger(atMs)||atMs<=previous||atMs>3500||
       !Number.isSafeInteger(keys)||keys<0||keys>255||
       (keys&3)===3||(keys&12)===12||
       !Number.isSafeInteger(dx)||dx< -127||dx>127||
       !Number.isSafeInteger(dy)||dy< -127||dy>127||
       ttlMs!==350) {
      throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
    }
    previous=atMs;
    args.push({atMs,keys,dx,dy,ttlMs});
  }
  // A valid action timeline may begin after a wait. Do not invent a
  // new 0ms packet: the native batch timer waits until the first
  // authored offset, and the controller remains idle beforehand.
  const serialized=JSON.stringify(args);
  if(Buffer.byteLength(serialized,'utf8')>8192) {
    throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  }
  let stdout;
  try {
    const response=await execFileAsync(executable,[
      '--lazy-tester-agent-cli','mc02-input-batch','2',
      String(Date.now()),serialized,
    ],{shell:false,windowsHide:true,timeout:9000,maxBuffer:8192,signal});
    stdout=response.stdout;
  } catch(error) {
    if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
    if(error.killed||error.signal) {
      throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    }
    if(typeof error.stdout==='string') {
      try {
        const parsed=JSON.parse(error.stdout.trim());
        if(parsed?.ok===false&&/^[A-Z_]{1,40}$/.test(parsed.code??'')) {
          throw new Error('Native request refused: '+parsed.code);
        }
      } catch(parsed) {
        if(parsed?.message?.startsWith('Native request refused:'))throw parsed;
      }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  let receipt;
  try {receipt=JSON.parse(stdout.trim());} catch {}
  if(receipt?.schema!==1||receipt?.ok!==true||
     receipt?.operation!=='mc02-input-batch'||receipt?.slot!==2||
     receipt?.pulses!==args.length||
     receipt?.delivery!=='PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF'||
     receipt?.gameplayObserved!==false) {
    throw new Error('MC02_INPUT_NATIVE_RESULT_UNVERIFIED');
  }
  return {schema:1,slot:2,
    status:'EXPERIMENTAL_MC02_SEQUENCE_TRANSPORT_ACKNOWLEDGED',
    pulsesAcknowledged:args.length,
    neutralState:'NATIVE_NEUTRAL_WRITE_ACKNOWLEDGED_ONLY',
    inputEffect:'NOT_VERIFIED_IN_MINECRAFT_EDUCATION',
    gameplayObserved:false,humanInputUninterrupted:'NOT_PROVEN',
    note:'One native CLI child reused retained controller IPC for every bounded MC02 pulse; no Minecraft gameplay was observed.'};
}

// The original MC02 batch CLI reports success only AFTER exact-channel
// native ReleaseAll has been acknowledged. A second neutral CLI may be
// backdated behind that accepted release, creating a false failure.
// Only this exact canonical sender may reuse native transport proof.
// Injected providers, aborts and uncertain results still need the
// independent original-channel neutral path. No game key-up proof.
export function canReuseVerifiedMc02BatchNeutral(receipt,pulseCount,{
  verifiedSender=false,transportConfirmed=false,
}={}) {
  return verifiedSender===true && transportConfirmed===true &&
    Number.isInteger(pulseCount)&&pulseCount>0&&pulseCount<=64 &&
    receipt?.schema===1 && receipt?.slot===2 &&
    receipt?.status==='EXPERIMENTAL_MC02_SEQUENCE_TRANSPORT_ACKNOWLEDGED' &&
    receipt?.pulsesAcknowledged===pulseCount &&
    receipt?.neutralState==='NATIVE_NEUTRAL_WRITE_ACKNOWLEDGED_ONLY' &&
    receipt?.inputEffect==='NOT_VERIFIED_IN_MINECRAFT_EDUCATION' &&
    receipt?.gameplayObserved===false;
}

// Only the canonical original native batch may prove that its own
// cleanup is obsolete OR that it could not pin the original child
// BEFORE dispatching ANY pulse. Neither allows a freshly timestamped
// fallback neutral against a possibly new MC02 process. Fake callbacks
// and ambiguous errors retain independent original-channel cleanup.
export function confirmedObsoleteMc02BatchCleanup(error,{
  verifiedSender=false,
}={}) {
  return verifiedSender===true && [
    'Native request refused: MC02_RELEASE_OBSOLETE',
    'Native request refused: MC02_BATCH_TARGET_UNVERIFIED',
  ].includes(error?.message);
}

// One opaque, ephemeral writer correlation ID per explicitly opted-in MCP.
// It is not OS authentication or a game credential; the retained controller
// arbitrates all actual per-original-game input. Never expose it over MCP
// or through Windows command-line arguments.
const AI_WRITER_ID_PATTERN=/^[a-f0-9]{32}$/;
// Versioned, transaction-local epoch without an IPC schema change.
// First 16 hex chars encode Windows FILETIME at MCP issuance (100-ns
// ticks since 1601); next 14 are fresh entropy; 'e1' is a format tag.
// This is temporal fencing, NOT authentication of a same-user process.
// The original Rust controller rechecks age and every child's PID/birth.
const AI_EPOCH_VERSION='e1';
const UNIX_TO_FILETIME_MS=11644473600000n;
export function makeAiLeaseEpoch(
    unixMs=performance.timeOrigin+performance.now(),
    entropy=randomBytes(7)) {
  if(typeof unixMs!=='number'||!Number.isFinite(unixMs)||unixMs<0||
     !Number.isSafeInteger(Math.floor(unixMs))||
     !Buffer.isBuffer(entropy)||entropy.length!==7) {
    throw new Error('AI_LEASE_EPOCH_REQUIRED');
  }
  const millis=Math.floor(unixMs);
  const subMillisecondTicks=Math.floor((unixMs-millis)*10000);
  const born=(BigInt(millis)+UNIX_TO_FILETIME_MS)*10000n+
    BigInt(subMillisecondTicks);
  if(born<=0n||born>0xffffffffffffffffn) {
    throw new Error('AI_LEASE_EPOCH_REQUIRED');
  }
  return born.toString(16).padStart(16,'0')+
    entropy.toString('hex')+AI_EPOCH_VERSION;
}
// Only an exact native writer conflict proves that acquisition never
// succeeded. A transport timeout, malformed receipt, or generic refusal
// must remain uncertain until explicit owner-side reconciliation.
export function definitelyDeniedWriterLease(error) {
  return [
    'Native request refused: NATIVE_WRITER_BUSY',
    'Native request refused: NATIVE_WRITER_MODE_MISMATCH',
    // Rust ai_lease_acquire validates temporal epoch and ALL original
    // process births before any AiLeaseBook mutation. This fixed exact
    // refusal therefore cannot have dispatched or acquired a writer.
    'Native request refused: NATIVE_WRITER_EPOCH_STALE',
  ].includes(error?.message);
}
// A newly acquired lease has a fresh nonce even when the same MCP
// process later controls the same slot. A delayed prior CLI packet
// cannot cross this native-actor fencing boundary after exit/relaunch.
function writerEnvironment(writerId,mode,leaseEpoch){
  if(typeof writerId!=='string'||!AI_WRITER_ID_PATTERN.test(writerId) ||
     !['human-plus-four-ai','five-ai'].includes(mode)) {
    throw new Error('AI_WRITER_LEASE_REQUIRED');
  }
  if(typeof leaseEpoch!=='string'||!AI_WRITER_ID_PATTERN.test(leaseEpoch)||
     !leaseEpoch.endsWith(AI_EPOCH_VERSION)){
    throw new Error('AI_LEASE_EPOCH_REQUIRED');
  }
  return {...process.env,LAZY_TESTER_AI_WRITER_ID:writerId,
    LAZY_TESTER_AI_WRITER_MODE:mode,
    LAZY_TESTER_AI_LEASE_EPOCH:leaseEpoch};
}
export async function sendExperimentalAiWriterLeaseNative(
    executable,operation,slots,
    {writerId,mode,leaseEpoch,neutralOnly=false}={}){
  if(!executable||!path.isAbsolute(executable)||
     !['acquire','release'].includes(operation)||
     !Array.isArray(slots)||slots.length<1||slots.length>5||
     slots.some((slot,i)=>!Number.isInteger(slot)||slot<1||slot>5||
       (i>0&&slots[i-1]>=slot))) {
    throw new Error('AI_WRITER_LEASE_INVALID');
  }
  if(typeof neutralOnly!=='boolean')throw new Error('AI_WRITER_LEASE_INVALID');
  const env=writerEnvironment(writerId,mode,leaseEpoch);
  const name='ai-lease-'+operation;
  let stdout;
  try{
    const result=await execFileAsync(executable,[
      '--lazy-tester-agent-cli',name,slots.join(','),
      ...(operation==='acquire'?[neutralOnly?'recovery':'action']:[]),
    ],{shell:false,windowsHide:true,timeout:6000,maxBuffer:8192,env});
    stdout=result.stdout;
  }catch(error){
    if(error.killed||error.signal){
      throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    }
    if(typeof error.stdout==='string'){
      try{
        const parsed=JSON.parse(error.stdout.trim());
        if(parsed?.ok===false&&/^[A-Z_]{1,40}$/.test(parsed.code??'')) {
          throw new Error('Native request refused: '+parsed.code);
        }
      }catch(parsed){
        if(parsed?.message?.startsWith('Native request refused:'))throw parsed;
      }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  let receipt;
  try{receipt=JSON.parse(stdout.trim());}catch{}
  if(receipt?.schema!==1||receipt?.ok!==true||
     receipt?.operation!==name||
     JSON.stringify(receipt?.slots)!==JSON.stringify(slots)||
     receipt?.lease!=='NATIVE_CONTROLLER_PER_SLOT_ONLY'||
     receipt?.gameplayObserved!==false) {
    throw new Error('AI_WRITER_LEASE_UNCONFIRMED');
  }
  return {schema:1,slots,status:operation==='acquire'?
    'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
    gameplayObserved:false,proof:'NATIVE_CONTROLLER_ONLY_NOT_GAMEPLAY'};
}

// One short-lived SAME-IMAGE CLI coordinates all requested AI slots.
// No second Minecraft session owner or background agent daemon is created.
// The native transport receipt must be tied to the exact ordered
// pulse bytes (including keys, mouse look and TTL), not just the number
// of slots/timestamps. SHA-256 is NOT game proof or CLI authentication.
export function matchesNativeMultiClientBatchReceipt(receipt,{
  slots,pulseCount,timestampGroups,pulseDigest,
}={}) {
  return receipt?.schema===1 && receipt?.ok===true &&
    receipt?.operation==='ai-multi-input-batch' &&
    Array.isArray(slots) && slots.length>0 && slots.length<=5 &&
    slots.every((slot,i)=>Number.isInteger(slot)&&slot>=1&&slot<=5&&
      (i===0||slot>slots[i-1])) &&
    Array.isArray(receipt?.slots) &&
    JSON.stringify(receipt.slots)===JSON.stringify(slots) &&
    receipt?.pulses===pulseCount &&
    receipt?.timestampGroups===timestampGroups &&
    typeof pulseDigest==='string' && /^[a-f0-9]{64}$/.test(pulseDigest) &&
    receipt?.pulseDigest===pulseDigest &&
    receipt?.neutralState==='NATIVE_PER_SLOT_NEUTRAL_ACK' &&
    JSON.stringify(receipt?.neutralSlots)===JSON.stringify(slots) &&
    receipt?.delivery==='PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF' &&
    receipt?.gameplayObserved===false;
}

// Only a successful receipt from the EXACT original verified Node
// sender may replace redundant MCP neutral writes. Test-injected native
// callbacks are deliberately NOT authoritative: they must still invoke
// the separate per-slot neutral path, even if they spoof these fields.
// This proves shared-state neutral ACK, NOT a Minecraft key-up.
export function canReuseVerifiedNativeBatchNeutral(receipt,slots,{
  verifiedSender=false,transportConfirmed=false,
}={}) {
  return verifiedSender===true && transportConfirmed===true &&
    Array.isArray(slots) && slots.length>0 && slots.length<=5 &&
    slots.every((slot,i)=>Number.isInteger(slot)&&slot>=1&&slot<=5&&
      (i===0||slot>slots[i-1])) &&
    receipt?.schema===1 && receipt?.gameplayObserved===false &&
    receipt?.status==='EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED' &&
    receipt?.nativeNeutralState==='NATIVE_PER_SLOT_NEUTRAL_ACK' &&
    JSON.stringify(receipt?.nativeNeutralSlots)===JSON.stringify(slots) &&
    typeof receipt?.pulseDigest==='string' &&
    /^[a-f0-9]{64}$/.test(receipt.pulseDigest);
}

export async function sendExperimentalMultiClientBatchNative(
    executable,batches,{signal,writerId,mode,leaseEpoch}={}) {
  if(!executable||!path.isAbsolute(executable)||!Array.isArray(batches)||
     batches.length<1||batches.length>5) {
    throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
  }
  const slots=batches.map(b=>b.slot);
  if(new Set(slots).size!==slots.length ||
     slots.some(slot=>!Number.isInteger(slot)||slot<1||slot>5)) {
    throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
  }
  const pulses=batches.flatMap(batch=>{
    if(!Array.isArray(batch.pulses)||batch.pulses.length<1||
       batch.pulses.length>64) {
      throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
    }
    return batch.pulses.map(pulse=>{
      const {atMs,keys,dx,dy,ttlMs}=pulse;
      if(!Number.isSafeInteger(atMs)||atMs<0||atMs>3500||
         !Number.isSafeInteger(keys)||keys<0||keys>255||
         (keys&3)===3||(keys&12)===12||
         !Number.isSafeInteger(dx)||dx< -127||dx>127||
         !Number.isSafeInteger(dy)||dy< -127||dy>127||
         ttlMs!==350) {
        throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
      }
      return {slot:batch.slot,atMs,keys,dx,dy,ttlMs};
    });
  }).sort((a,b)=>a.atMs-b.atMs||a.slot-b.slot);
  for(let i=1;i<pulses.length;i++) {
    if(pulses[i].slot===pulses[i-1].slot &&
       pulses[i].atMs===pulses[i-1].atMs) {
      throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
    }
  }
  // Native now processes all slots sharing one timeline timestamp in one
  // authenticated IPC group. Never call this concurrent/simultaneous game
  // input: the retained actor still revalidates and writes each channel.
  const expectedGroups=new Set(pulses.map(pulse=>pulse.atMs)).size;
  const serialized=JSON.stringify(pulses);
  if(Buffer.byteLength(serialized,'utf8')>24000) {
    throw new Error('MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS');
  }
  // Exact bytes passed to the original same-image native CLI.
  const pulseDigest=createHash('sha256').update(serialized,'utf8').digest('hex');
  let stdout;
  try {
    const response=await execFileAsync(executable,[
      '--lazy-tester-agent-cli','ai-multi-input-batch',
      String(Date.now()),serialized,
    ],{shell:false,windowsHide:true,timeout:9000,maxBuffer:8192,signal,
      env:writerEnvironment(writerId,mode,leaseEpoch)});
    stdout=response.stdout;
  }catch(error){
    if(signal?.aborted)throw new Error('AI_SEQUENCE_ABORTED');
    if(error.killed||error.signal) {
      throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
    }
    if(typeof error.stdout==='string') {
      try {
        const output=JSON.parse(error.stdout.trim());
        if(output?.ok===false&&/^[A-Z_]{1,40}$/.test(output.code??'')) {
          throw new Error('Native request refused: '+output.code);
        }
      }catch(parsed){
        if(parsed?.message?.startsWith('Native request refused:'))throw parsed;
      }
    }
    throw new Error('Native controller unavailable or operation refused');
  }
  let receipt;
  try{receipt=JSON.parse(stdout.trim());}catch{}
  if(!matchesNativeMultiClientBatchReceipt(receipt,{
    slots:[...slots].sort((a,b)=>a-b),pulseCount:pulses.length,
    timestampGroups:expectedGroups,pulseDigest,
  })) {
    // Dispatch may have reached one or all native slots. Keep the
    // caller's existing per-slot neutral/quarantine path; never replay.
    throw new Error('AI_INPUT_NATIVE_RESULT_UNVERIFIED');
  }
  return {schema:1,slots:[...slots].sort((a,b)=>a-b),
    pulsesAcknowledged:pulses.length,timestampGroupsAcknowledged:expectedGroups,
    pulseDigest,
    nativeNeutralState:receipt.neutralState,
    nativeNeutralSlots:[...receipt.neutralSlots],
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    gameplayObserved:false,
    humanInputUninterrupted:'NOT_PROVEN',
    inputEffect:'NOT_VERIFIED_IN_MINECRAFT_EDUCATION',
    note:'A single bounded native scheduler wrote separate per-slot input state; no actual game response has been verified.'};
}

