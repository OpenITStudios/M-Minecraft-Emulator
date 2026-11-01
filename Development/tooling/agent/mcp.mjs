#!/usr/bin/env node
// Local stdio MCP bridge. No HTTP listener, shell commands, credential access or native game input.
// Every native mutation goes through the existing authenticated Lazy Tester controller.
import {execFile} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstat, open} from 'node:fs/promises';
import {pathToFileURL} from 'node:url';
import path from 'node:path';
import readline from 'node:readline';
import {promisify} from 'node:util';
import {planPeerSession,planPlayerActions} from './control-plan.mjs';
import {compilePlayerInputTimeline,compilePerClientInputFrames,GAME_BUTTON_BITS} from './input-timeline.mjs';
import {assessInputRoute,requiredInputSlots} from './input-route.mjs';
import {inputExecutionStatus} from './input-delivery.mjs';
import {prepareMc02PulseBatch,prepareMultiClientPulseBatches} from './mc02-action-sequence.mjs';
import {summarizeGameEvidence,summarizeEvidenceReports,normalizeEvidenceSlots} from './evidence-summary.mjs';
import {captureCuaWindow,validateCuaExecutable} from './visual-capture.mjs';
import {createVisualObserver,OBSERVATION_LIMITS,
  summarizeTeamVisualState,verifyTeamObservationBasis,
  evaluateTeamActionFeedback,summarizeTeamCoordination} from './visual-observer.mjs';
import {createGpuFrameSource,validateGpuExecutable} from './gpu-frame-source.mjs';

const execFileAsync = promisify(execFile);
const REQUEST_LIMIT = 64 * 1024;
const MAX_MCP_TEXT_BYTES = 128 * 1024;
const MAX_MCP_IMAGE_CHARS = 8 * 1024 * 1024; // Cua's 6 MiB PNG base64 upper bound
const EVIDENCE_TAIL_BYTES = 256 * 1024;
const EVIDENCE_LIMIT = 20;
const VALID_STATES = new Set(['STOPPED','STARTING','RUNNING','NOT_RESPONDING','ATTENTION']);
const SUPPORTED_PROTOCOLS = new Set(['2025-06-18','2025-11-25','2026-07-28']);

// Only fixed application errors are exposed over MCP. Filesystem and
// provider messages may contain account paths, native diagnostics or
// untrusted log text. Internal direct callTool() errors remain available
// to source tests; these do not become model-visible error strings.
const PUBLIC_TOOL_ERRORS=new Set([
  'Unknown tool','Invalid tool arguments',
  'slot must be an integer from 1 to 5',
  'Native operation in progress; do not overlap requests',
  'MC01 is reserved for the human host in hybrid mode',
  'MC01 human host is protected from AI visual observation',
  'MC01 human host is protected from AI computer-use targeting',
  'Visual provider is NOT_CONFIGURED',
  'Visual provider is NOT_CONFIGURED; set --cua-exe or --gpu-capture-exe',
  'Native integration is not configured; set --exe',
  'Native controller unavailable or operation refused',
  'Native request outcome UNCERTAIN; do not retry blindly',
  'GPU_OBSERVATION_START_CANCELLED',
  'MCP_TEXT_RESPONSE_TOO_LARGE','MCP_IMAGE_RESPONSE_TOO_LARGE',
  'MC02_EXPERIMENT_DISABLED','MC02_INPUT_INVALID_COMMAND',
  'MC02_INPUT_HUMAN_FOREGROUND_REQUIRED',
  'MC02_INPUT_TARGETS_NOT_SEPARATE','MC02_INPUT_NATIVE_RESULT_UNVERIFIED',
  'MC02_INPUT_REQUEST_STALE',
  'MC02_SEQUENCE_INVALID_ACTIONS','MC02_SEQUENCE_NATIVE_UNCONFIRMED',
  'MC02_SEQUENCE_DEADLINE','MC02_SEQUENCE_ABORTED','MC02_RELEASE_UNCONFIRMED',
  'MULTI_CLIENT_EXPERIMENT_DISABLED','MULTI_CLIENT_SEQUENCE_INVALID_ACTIONS',
  'TEAM_OBSERVATION_INCOMPLETE','TEAM_OBSERVATION_STALE_OR_REPLACED',
  'TEAM_OBSERVATION_TIMING_UNAVAILABLE','TEAM_ACTION_RECEIPT_MISMATCH',
  'AI_SEQUENCE_ABORTED','AI_RELEASE_UNCONFIRMED','AI_INPUT_NATIVE_RESULT_UNVERIFIED',
  'AI_INPUT_RECOVERY_REQUIRED',
]);
function publicToolError(error) {
  const message=typeof error?.message==='string' ? error.message : '';
  return PUBLIC_TOOL_ERRORS.has(message) ||
    /^Native request refused: [A-Z_]{1,40}$/.test(message) ?
    message : 'Tool unavailable; details withheld';
}
export function boundedMcpText(value) {
  const encoded=JSON.stringify(value);
  if(typeof encoded!=='string' ||
     Buffer.byteLength(encoded,'utf8')>MAX_MCP_TEXT_BYTES) {
    throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
  }
  return encoded;
}

export function parseConfiguration(argv, env = process.env) {
  let mode = 'human-plus-four-ai';
  let executable = env.LAZY_TESTER_AGENT_EXE ?? '';
  let evidenceDirectory = env.LAZY_TESTER_EVIDENCE_DIR ?? '';
  let cuaExecutable = env.LAZY_TESTER_CUA_EXE ?? '';
  let gpuExecutable = env.LAZY_TESTER_GPU_CAPTURE_EXE ?? '';
  let experimentalMc02InputEnabled=false;
  let experimentalMultiClientInputEnabled=false;
  for (let i=0;i<argv.length;i++) {
    const flag=argv[i];
    if (!['--mode','--exe','--evidence-dir','--cua-exe','--gpu-capture-exe','--mc02-input-experimental','--multi-client-input-experimental'].includes(flag) || i===argv.length-1) {
      throw new Error('Accepted arguments: --mode human-plus-four-ai|five-ai --exe <lazy-tester.exe> [--evidence-dir <folder>] [--cua-exe <cua-driver.exe>] [--gpu-capture-exe <lazy-tester-gpu-capture.exe>]');
    }
    const value=argv[++i];
    if (flag==='--mode') mode=value;
    if (flag==='--exe') executable=value;
    if (flag==='--evidence-dir') evidenceDirectory=value;
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
  if (cuaExecutable) validateCuaExecutable(cuaExecutable);
  if (gpuExecutable) validateGpuExecutable(gpuExecutable);
  return Object.freeze({mode, executable, evidenceDirectory,
    cuaExecutable,gpuExecutable,experimentalMc02InputEnabled,
    experimentalMultiClientInputEnabled});
}

function slotOf(value) {
  if (!Number.isInteger(value) || value < 1 || value > 5) throw new Error('slot must be an integer from 1 to 5');
  return value;
}
function keysExactly(value, allowed) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).some(key=>!allowed.includes(key))) throw new Error('Invalid tool arguments');
  return value;
}
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

function computerUseTarget(value,slot,mode) {
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
function mc02ButtonMask(buttons) {
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
    {slot,keys,dx,dy,ttlMs,requestedAtMs},{legacy=false}={}) {
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
    ],{shell:false,windowsHide:true,timeout:6000,maxBuffer:8192});
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

// One short-lived SAME-IMAGE CLI coordinates all requested AI slots.
// No second Minecraft session owner or background agent daemon is created.
export async function sendExperimentalMultiClientBatchNative(
    executable,batches,{signal}={}) {
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
  let stdout;
  try {
    const response=await execFileAsync(executable,[
      '--lazy-tester-agent-cli','ai-multi-input-batch',
      String(Date.now()),serialized,
    ],{shell:false,windowsHide:true,timeout:9000,maxBuffer:8192,signal});
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
  if(receipt?.schema!==1||receipt?.ok!==true||
     receipt?.operation!=='ai-multi-input-batch'||
     !Array.isArray(receipt?.slots)||
     JSON.stringify(receipt.slots)!==JSON.stringify([...slots].sort((a,b)=>a-b))||
     receipt?.pulses!==pulses.length||
     receipt?.timestampGroups!==expectedGroups||
     receipt?.delivery!=='PER_CLIENT_SHARED_STATE_WRITTEN_NOT_GAMEPLAY_PROOF'||
     receipt?.gameplayObserved!==false) {
    throw new Error('AI_INPUT_NATIVE_RESULT_UNVERIFIED');
  }
  return {schema:1,slots:[...slots].sort((a,b)=>a-b),
    pulsesAcknowledged:pulses.length,timestampGroupsAcknowledged:expectedGroups,
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    gameplayObserved:false,
    humanInputUninterrupted:'NOT_PROVEN',
    inputEffect:'NOT_VERIFIED_IN_MINECRAFT_EDUCATION',
    note:'A single bounded native scheduler wrote separate per-slot input state; no actual game response has been verified.'};
}

export function parseCapturedLine(line) {
  const match=/^(\d{4}-\d\d-\d\dT\S+)\s+(verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if (!match || match[1].length>45 ||
      !Number.isFinite(Date.parse(match[1]))) return null;
  // Debug-log-capture stores JSON-string-escaped PrintEvent text without
  // the enclosing quotes; preserve legacy raw-line fixtures as well.
  let message=match[3];
  try { message=JSON.parse('"'+message+'"'); }
  catch { /* The original capture may already contain unescaped text. */ }

  // A mention inside ordinary console text is NOT a Skills Probe event.
  // Accept exactly one leading source marker followed by one JSON object.
  const tagged=/^\s*(\[ANALYTICS\]|\[GAME_RESULTS\])\s*(\{[\s\S]*)$/.exec(message);
  if (!tagged || Buffer.byteLength(tagged[2],'utf8')>16*1024) return null;
  let data;
  try { data=JSON.parse(tagged[2]); } catch { return null; }
  if (!data || typeof data!=='object' || Array.isArray(data)) return null;
  const analytics=tagged[1]==='[ANALYTICS]';
  const kind=analytics?data.type:data.record_type;
  if (typeof kind!=='string' || !kind.trim() || kind.length>80 ||
      (data.schema!==undefined && data.schema!==1)) return null;
  // GAME_RESULTS is an untrusted marker; only the currently recognized
  // v1 gameplay-result shape may be counted as a result, never as PASS.
  if (!analytics &&
      (data.schema!==1 || kind!=='gameplay_result')) return null;
  return {
    observedAt:match[1],level:match[2],
    type:analytics?'analytics':'game_results',data,
  };
}

// Stable evidence snapshots are a read-only best effort, not a game/player
// identity attestation. Reject unidentifiable filesystems instead of using
// a size/mtime guess when a log can be replaced between lstat and open.
function sameEvidenceFile(a,b) {
  return a && b &&
    typeof a.dev==='bigint' && typeof b.dev==='bigint' &&
    typeof a.ino==='bigint' && a.ino!==0n &&
    typeof b.ino==='bigint' && b.ino!==0n &&
    a.dev===b.dev && a.ino===b.ino &&
    a.isFile() && b.isFile() &&
    !a.isSymbolicLink() && !b.isSymbolicLink();
}
function sameEvidenceSnapshot(a,b) {
  return sameEvidenceFile(a,b) &&
    typeof a.size==='bigint' && typeof b.size==='bigint' &&
    typeof a.mtimeNs==='bigint' && typeof b.mtimeNs==='bigint' &&
    typeof a.ctimeNs==='bigint' && typeof b.ctimeNs==='bigint' &&
    a.size===b.size && a.mtimeNs===b.mtimeNs &&
    a.ctimeNs===b.ctimeNs;
}
function unstableEvidence(slot,reason) {
  return {slot,status:'CAPTURE_FILE_UNSTABLE',events:[],
    attribution:'UNVERIFIED',reason};
}

export async function readGameEvidence(directory,slot,maxRecords=EVIDENCE_LIMIT,{
  lstatFile=lstat,openFile=open,
}={}) {
  slotOf(slot);
  if (!directory) return {slot,status:'NOT_CONFIGURED',events:[],attribution:'UNVERIFIED'};
  const file=path.join(directory,'MC0'+slot+'.log');
  let observedPath=false;
  try {
    const parent=await lstatFile(directory);
    if (!parent.isDirectory() || parent.isSymbolicLink()) {
      throw new Error('Evidence folder must be an ordinary directory');
    }
    const namedBefore=await lstatFile(file,{bigint:true});
    observedPath=true;
    if (!namedBefore.isFile() || namedBefore.isSymbolicLink()) {
      throw new Error('Evidence file is not an ordinary file');
    }
    const handle=await openFile(file,'r');
    try {
      // Use this already-open handle's size/identity for offsets, not a
      // pathname stat which may now refer to a rotated/replaced log.
      const pinned=await handle.stat({bigint:true});
      if (!sameEvidenceSnapshot(namedBefore,pinned)) {
        return unstableEvidence(slot,'FILE_CHANGED_DURING_READ');
      }
      if (pinned.size>BigInt(Number.MAX_SAFE_INTEGER)) {
        return unstableEvidence(slot,'FILE_OFFSET_UNREPRESENTABLE');
      }
      const bytes=Number(pinned.size>BigInt(EVIDENCE_TAIL_BYTES) ?
        BigInt(EVIDENCE_TAIL_BYTES):pinned.size);
      const start=Number(pinned.size-BigInt(bytes));
      const buffer=Buffer.alloc(bytes);
      let offset=0;
      while (offset<bytes) {
        // A positional regular-file read may return fewer bytes than
        // requested; never silently parse a short or torn snapshot.
        const {bytesRead}=await handle.read(buffer,offset,bytes-offset,start+offset);
        if (!Number.isSafeInteger(bytesRead) || bytesRead<=0 ||
            bytesRead>bytes-offset) {
          return unstableEvidence(slot,'INCOMPLETE_FILE_READ');
        }
        offset+=bytesRead;
      }
      const handleAfter=await handle.stat({bigint:true});
      const namedAfter=await lstatFile(file,{bigint:true});
      if (!sameEvidenceSnapshot(pinned,handleAfter) ||
          !sameEvidenceSnapshot(pinned,namedAfter)) {
        return unstableEvidence(slot,'FILE_CHANGED_DURING_READ');
      }
      const data=buffer.toString('utf8');
      const entries=data.split(/\r?\n/);
      if (start>0) entries.shift(); // Offset may cut through first line.
      const incompleteTrailingLineIgnored=
        data.length>0 && !data.endsWith('\n');
      if (incompleteTrailingLineIgnored) entries.pop();
      const records=entries.map(parseCapturedLine).filter(Boolean);
      return {slot,status:'CAPTURE_FILE_PRESENT',
        events:records.slice(-Math.min(maxRecords,EVIDENCE_LIMIT)),
        attribution:'OPERATOR_NAMED_UNVERIFIED',
        truncated:pinned.size>BigInt(EVIDENCE_TAIL_BYTES),
        incompleteTrailingLineIgnored,
        note:'Log markers are untrusted. Stable file bytes do not prove account, player, world or gameplay identity.'};
    } finally {await handle.close();}
  } catch(error) {
    if (error?.code==='ENOENT') {
      // Missing before first observation is ordinary absence. Disappearing
      // after first lstat is log rotation and must never return old events.
      return observedPath ? unstableEvidence(slot,'FILE_DISAPPEARED_DURING_READ') :
        {slot,status:'NO_CAPTURE_FILE',events:[],attribution:'UNVERIFIED'};
    }
    throw error;
  }
}

export const TOOL_LIST=[
  {name:'start_visual_observation',description:'Start a time-limited LOCAL read-only frame sampler for 1-5 AI-owned Minecraft windows. No continuous images are sent to the model and no gameplay inputs occur.',inputSchema:{type:'object',properties:{slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,items:{type:'integer',minimum:1,maximum:5}},intervalMs:{type:'integer',minimum:400,maximum:5000},durationMs:{type:'integer',minimum:5000,maximum:90000}},required:['slots'],additionalProperties:false}},
  {name:'get_multi_client_visual_state',
    description:'Read a single compact METADATA-ONLY view of existing cached visual frames across 1-5 AI clients. No screenshot is taken, no pixels are transmitted, and changed PNG bytes are not gameplay proof. Read individual frames explicitly to make model decisions.',
    inputSchema:{type:'object',properties:{
      slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
        items:{type:'integer',minimum:1,maximum:5}},
      since:{type:'array',minItems:1,maxItems:5,items:{type:'object',
        properties:{
          slot:{type:'integer',minimum:1,maximum:5},
          sequence:{type:'integer',minimum:0},
          samples:{type:'integer',minimum:0},
          collectedAt:{type:'string'},
          sourceFrameAtMs:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
          sourceTimeKind:{enum:['WGC_READBACK_COMPLETED','CUA_QUERY_STARTED',null]},
          producerSequence:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
          captureIdentity:{type:'object',properties:{
            pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},
            sessionId:{type:'integer',minimum:0},createdFileTime:{type:'string'},
          },required:['pid','windowId','sessionId','createdFileTime'],
            additionalProperties:false},
        },required:['slot','sequence','samples','collectedAt',
          'sourceFrameAtMs','sourceTimeKind','producerSequence','captureIdentity'],
        additionalProperties:false}},
    },additionalProperties:false}},
  {name:'read_visual_observation',description:'Retrieve only a NEW locally cached frame (and status), or metadata without pixels. Never triggers a screenshot itself.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},afterSequence:{type:'integer',minimum:0},includeImage:{type:'boolean'}},required:['slot'],additionalProperties:false}},
  {name:'stop_visual_observation',description:'Stop one or more bounded visual samplers and immediately drop their in-memory image caches.',inputSchema:{type:'object',properties:{slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,items:{type:'integer',minimum:1,maximum:5}}},required:['slots'],additionalProperties:false}},
  {name:'observe_client',description:'Read-only screenshot of one AI-owned native Minecraft window via an explicitly configured Cua Driver daemon; rejects a replaced or stale window, never takes foreground or sends input.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'get_computer_use_target',description:'Resolve one AI-owned native Minecraft PID/HWND for read-only Cua Driver window capture without switching Windows focus. Does NOT capture pixels or control game input.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'revalidate_computer_use_target',description:'Check a previously resolved exact PID/HWND/birth identity against current retained-controller observation. Never silently retarget another game.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},createdFileTime:{type:'string',pattern:'^[0-9]{1,20}$'}},required:['slot','pid','windowId','createdFileTime'],additionalProperties:false}},
  {name:'get_test_context',description:'One bounded read-only handoff with current five native profiles, peer-host guidance and relevant game evidence. No game control or world success is inferred.',inputSchema:{type:'object',properties:{slots:{type:'array',items:{type:'integer',minimum:1,maximum:5},minItems:1,maxItems:5,uniqueItems:true}},additionalProperties:false}},
  {name:'agent_capabilities',description:'Read capabilities and limits of this Windows five-profile test bridge. Never claims independent background movement.',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'inspect_profiles',description:'Read five profile states from the authenticated native Lazy Tester controller (not game login or world readiness).',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'launch_profile',description:'Explicitly request native Minecraft launch for an AI-controlled profile. Human MC01 cannot be launched in hybrid mode.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'plan_peer_session',description:'Inspect native profiles and provide conservative human/AI host-and-join guidance. No world control or join is performed.',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'plan_player_actions',description:'Validate bounded concurrent AI player-action intents. PREVIEW ONLY: no input is sent to Minecraft.',inputSchema:{type:'object',properties:{actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},kind:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use','look_left','look_right','look_relative','wait']},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:5000},deltaX:{type:'integer',minimum:-512,maximum:512},deltaY:{type:'integer',minimum:-512,maximum:512}},required:['slot','kind','atMs','durationMs'],additionalProperties:false}},includeTimeline:{type:'boolean'},includeClientFrames:{type:'boolean'}},required:['actions'],additionalProperties:false}},
  {name:'assess_input_route',description:'Read-only native identity check for AI gameplay input: bind validated multi-client action timeline to real MC HWND/PID; ALWAYS reports no verified independent game-input backend and NEVER sends input.',inputSchema:{type:'object',properties:{actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},kind:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use','look_left','look_right','look_relative','wait']},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:5000},deltaX:{type:'integer',minimum:-512,maximum:512},deltaY:{type:'integer',minimum:-512,maximum:512}},required:['slot','kind','atMs','durationMs'],additionalProperties:false}}},required:['actions'],additionalProperties:false}},
  {name:'summarize_test_evidence',description:'Read and condense bounded analytics/game-results capture from 1-5 profiles. No login, player mapping, host/join or gameplay result is inferred.',inputSchema:{type:'object',properties:{slots:{type:'array',items:{type:'integer',minimum:1,maximum:5},minItems:1,maxItems:5,uniqueItems:true}},additionalProperties:false}},
  {name:'read_game_evidence',description:'Read only a compact allowlisted projection of 1-20 captured evidence records for one profile. No raw Skills Probe JSON, credentials or verified gameplay claims.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
];


// Deliberately NOT in TOOL_LIST: omitted from tools/list in the normal mode.
// Only an explicitly opted-in development MCP advertises it.
const EXPERIMENTAL_MC02_TOOL={
  name:'send_experimental_mc02_input',
  description:'EXPERIMENTAL/OPT-IN: send one <=500ms MC02-only virtual Win32 polling state. MC01 stays human foreground. This does not prove Minecraft gameplay or support Raw Input/GameInput.',
  inputSchema:{type:'object',properties:{
    slot:{type:'integer',const:2},
    buttons:{type:'array',maxItems:8,uniqueItems:true,
      items:{type:'string',enum:['move_forward','move_backward','strafe_left',
        'strafe_right','jump','sneak','attack','use']}},
    lookDX:{type:'integer',minimum:-127,maximum:127},
    lookDY:{type:'integer',minimum:-127,maximum:127},
    ttlMs:{type:'integer',minimum:1,maximum:500},
  },required:['slot','buttons'],additionalProperties:false},
};
// These are intentionally opt-in ONLY, using the same MC02 native shared-state
// writer. No provider for MC03-MC05 and no change to the normal tools list.
const EXPERIMENTAL_MC02_SEQUENCE_TOOLS=[
  {name:'run_experimental_mc02_actions',
    description:'EXPERIMENTAL MC02 only: execute a short timed action sequence with held-key heartbeat and best-effort neutral release. Native state ACK, NOT verified Minecraft gameplay.',
    inputSchema:{type:'object',properties:{
      slot:{type:'integer',const:2},
      actions:{type:'array',minItems:1,maxItems:12,
        items:{type:'object',properties:{
          slot:{type:'integer',const:2},
          kind:{type:'string',enum:['move_forward','move_backward',
            'strafe_left','strafe_right','jump','sneak','attack','use','look_relative']},
          atMs:{type:'integer',minimum:0,maximum:3500},
          durationMs:{type:'integer',minimum:50,maximum:2000},
          deltaX:{type:'integer',minimum:-127,maximum:127},
          deltaY:{type:'integer',minimum:-127,maximum:127},
        },required:['slot','kind','atMs','durationMs'],additionalProperties:false}},
    },required:['slot','actions'],additionalProperties:false}},
  {name:'stop_experimental_mc02_actions',
    description:'Cancel an active experimental MC02 sequence; when idle, request an exact-owner native neutral-state write. Transport ACK only, not gameplay release proof.',
    inputSchema:{type:'object',properties:{slot:{type:'integer',const:2}},
      required:['slot'],additionalProperties:false}},
];

// Registered ONLY after a separate developer opt-in. Native DLL/Cargo
// features remain absent from the ordinary packaged application.
const EXPERIMENTAL_MULTI_CLIENT_TOOLS=[
  {name:'run_experimental_multi_client_actions',
    description:'EXPERIMENTAL: dispatch a <=3.5s cross-slot action timeline to 1-5 opted-in AI game clients in one bounded native CLI. Transport acknowledgement only, NOT verified Minecraft gameplay.',
    inputSchema:{type:'object',properties:{
      actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{
        slot:{type:'integer',minimum:1,maximum:5},
        kind:{type:'string',enum:['move_forward','move_backward',
          'strafe_left','strafe_right','jump','sneak','attack','use','look_relative']},
        atMs:{type:'integer',minimum:0,maximum:3500},
        durationMs:{type:'integer',minimum:50,maximum:2000},
        deltaX:{type:'integer',minimum:-127,maximum:127},
        deltaY:{type:'integer',minimum:-127,maximum:127},
      },required:['slot','kind','atMs','durationMs'],additionalProperties:false}},
    },required:['actions'],additionalProperties:false}},
  {name:'stop_experimental_multi_client_actions',
    description:'Cancel the whole active cross-slot experiment if a requested slot participates, or request exact-owner neutral writes for listed idle AI clients. Never sends global keyboard/mouse input.',
    inputSchema:{type:'object',properties:{
      slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
        items:{type:'integer',minimum:1,maximum:5}},
    },required:['slots'],additionalProperties:false}},
];

// This opt-in tool only binds a RECENT per-slot frame snapshot to the
// existing native batch call. It never makes AI decisions or bypasses the
// original game controller's independent input admission checks.
const EXPERIMENTAL_OBSERVED_TEAM_TOOL={
  name:'run_experimental_observed_actions',
  description:'EXPERIMENTAL: require exact recent visual baselines for every AI slot, then reuse the existing one-CLI native action batch. Follow with get_multi_client_visual_state to check newer samples. No automatic gameplay perception or success claim.',
  inputSchema:{type:'object',properties:{
    actions:EXPERIMENTAL_MULTI_CLIENT_TOOLS[0].inputSchema.properties.actions,
    basis:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
      items:{type:'object',properties:{
        slot:{type:'integer',minimum:1,maximum:5},
        sequence:{type:'integer',minimum:1},
        samples:{type:'integer',minimum:1},
        collectedAt:{type:'string'},
        sourceFrameAtMs:{type:'integer',minimum:1},
        sourceTimeKind:{type:'string',
          enum:['WGC_READBACK_COMPLETED','CUA_QUERY_STARTED']},
        producerSequence:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
        captureIdentity:{type:'object',properties:{
          pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},
          sessionId:{type:'integer',minimum:0},
          createdFileTime:{type:'string',pattern:'^[0-9]{1,20}$'},
        },required:['pid','windowId','sessionId','createdFileTime'],
          additionalProperties:false},
      },required:['slot','sequence','samples','collectedAt',
        'sourceFrameAtMs','sourceTimeKind','producerSequence','captureIdentity'],
        additionalProperties:false}},
  },required:['actions','basis'],additionalProperties:false},
};

// Optional read-only result classification. Input receipt and AI review are
// caller-supplied; neither is independently authenticated as gameplay proof.
// SHA-256 of the deterministic canonical pulse plan. Prevents accidental
// evaluation against a different action list; it does not authenticate an
// externally copied or fabricated receipt and proves no Minecraft effect.
export function fingerprintActions(mode,actions) {
  const plan=prepareMultiClientPulseBatches(mode,actions);
  const projected=plan.batches.map(batch=>({
    slot:batch.slot,pulses:batch.pulses.map(pulse=>({
      atMs:pulse.atMs,keys:pulse.keys,dx:pulse.dx,dy:pulse.dy,ttlMs:pulse.ttlMs,
    })),
  }));
  return createHash('sha256').update(JSON.stringify({mode,projected}))
    .digest('hex');
}

const EXPERIMENTAL_TEAM_FEEDBACK_TOOL={
  name:'evaluate_experimental_multi_client_outcomes',
  description:'EXPERIMENTAL READ-ONLY: classify per-client movement, camera, attack and interaction reports against exact newer same-window frame metadata. A model-reported visual change is NEVER verified Minecraft movement or attack success.',
  inputSchema:{type:'object',properties:{
    actions:EXPERIMENTAL_MULTI_CLIENT_TOOLS[0].inputSchema.properties.actions,
    receipt:{type:'object'},
    after:{type:'array',minItems:1,maxItems:5,items:{anyOf:[
      EXPERIMENTAL_OBSERVED_TEAM_TOOL.inputSchema.properties.basis.items,
      {type:'null'},
    ]}},
    reviews:{type:'array',maxItems:20,items:{type:'object',properties:{
      slot:{type:'integer',minimum:1,maximum:5},
      category:{type:'string',enum:['movement','camera','attack','interaction']},
      assessment:{type:'string',enum:[
        'VISIBLE_CHANGE_REPORTED','NO_VISIBLE_CHANGE_REPORTED','UNDETERMINED',
      ]},
      cue:{type:'string',maxLength:160},
    },required:['slot','category','assessment'],additionalProperties:false}},
  },required:['actions','receipt','after'],additionalProperties:false},
};

function capabilities(mode,experimental=false,multi=false) {return {schema:1,mode,hostSlot:1,humanSlots:mode==='human-plus-four-ai'?[1]:[],
  aiSlots:mode==='human-plus-four-ai'?[2,3,4,5]:[1,2,3,4,5],
  available:['start_visual_observation','read_visual_observation','get_multi_client_visual_state','stop_visual_observation','observe_client','get_computer_use_target','revalidate_computer_use_target','get_test_context','inspect_profiles','launch_profile','read_game_evidence','plan_peer_session','plan_player_actions','assess_input_route','summarize_test_evidence',...(experimental?[EXPERIMENTAL_MC02_TOOL.name,...EXPERIMENTAL_MC02_SEQUENCE_TOOLS.map(t=>t.name)]:[]),
    ...(multi?[...EXPERIMENTAL_MULTI_CLIENT_TOOLS.map(t=>t.name),
      EXPERIMENTAL_OBSERVED_TEAM_TOOL.name,
      EXPERIMENTAL_TEAM_FEEDBACK_TOOL.name]:[])],
  notAvailable:['parallel_independent_keyboard_mouse','automatic_world_host_join','send_minecraft_command'],
  visualCapture:'OPTIONAL_CUA_OR_PERSISTENT_WGC_SOURCE_NOT_RUNTIME_VERIFIED',
  independentInput:{state:'BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND',
    simultaneousIntentChannels:true,actualInputEnabled:false,
    experimentalMc02Win32PollingOnly:experimental?'OPTED_IN_NOT_GAMEPLAY_VERIFIED':'DISABLED',
    experimentalMultiClientNative:multi?'OPTED_IN_SOURCE_CANDIDATE_NOT_GAMEPLAY_VERIFIED':'DISABLED',
    requiresHumanFocusProtection:mode==='human-plus-four-ai',
    requiresPerClientEmergencyRelease:true,
    nativeSessionModel:'SAME_WINDOWS_SESSION_ONLY',
    windowsAgentWorkspace:'PREVIEW_NOT_INTEGRATED_NO_CROSS_SESSION_INPUT',
    transactionEngine:inputExecutionStatus()},
  note:'MCP tools control the native session owner and read evidence; they do NOT independently move multiple background Minecraft clients.'};}

export function createHandler(config,{
  native=runNative,evidence=readGameEvidence,capture=captureCuaWindow,
  mc02Input=sendExperimentalMc02Native,
  mc02Batch=sendExperimentalMc02BatchNative,
  aiInput=sendExperimentalAiNative,
  aiMultiBatch=sendExperimentalMultiClientBatchNative,
  gpuSourceFactory=createGpuFrameSource,
  visualObserverFactory=createVisualObserver,
}={}) {
  let inFlight=false; // mutating or exclusive native operation
  let visualReadBusy=false; // only one identity-pinned screenshot at a time
  let mc02SequenceAbort=null; // pending opted-in legacy MC02 request only
  let multiSequence=null; // one bounded cross-slot request, not a game owner
  // Volatile MCP-process fail-closed input markers, not another game
  // controller, OS channel owner, retry queue or account state registry.
  const uncertainAiSlots=new Set();
  let observedSequenceSlots=null; // one opted-in observed batch at a time
  const markAiUncertain=slots=>{
    for(const slot of slots)uncertainAiSlots.add(slot);
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
      const before=validateNativeWindowTarget(
        await native(config.executable,'target',slot),slot);
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
      const after=validateNativeWindowTarget(
        await native(config.executable,'target',slot),slot);
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
      return {
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
    } finally {
      visualReadBusy=false;
    }
  }
  const revokeObservedRound=slot=>{
    if(observedSequenceSlots?.includes(slot)) {
      // A single missing/revoked captured frame source invalidates
      // the entire SELECTED action round. Never silently proceed with
      // the remaining AI clients or start a retry.
      markAiUncertain(observedSequenceSlots);
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
  async function releaseAiSlots(slots) {
    // Fail-closed independent cleanup, SERIALIZED on the retained
    // controller. Do not flood its authenticated pipe with concurrent
    // short-lived one-shot writers. Attempt all slots despite failures.
    let allConfirmed=true;
    for(const slot of slots) {
      try {
        const receipt=await aiInput(config.executable,{
          slot,keys:0,dx:0,dy:0,ttlMs:1,requestedAtMs:Date.now(),
        });
        if(receipt?.schema!==1||receipt?.slot!==slot||
           receipt?.status!=='NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED'||
           receipt?.gameplayObserved!==false)allConfirmed=false;
      } catch {allConfirmed=false;}
    }
    return allConfirmed;
  }
  async function callTool(name,args) {
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
          const target=validateNativeWindowTarget(
            await native(config.executable,'target',slot),slot);
          if (startup.cancelled) throw new Error('GPU_OBSERVATION_START_CANCELLED');
          await gpuSource.start(slot,target,{durationMs,fps:5});
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
        config.mode,visual,[...uncertainAiSlots].sort((a,b)=>a-b),multiEnabled);
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
      keysExactly(args,['slot','afterSequence','includeImage']);
      const slot=slotOf(args.slot);
      protectedSlot(slot);
      return visualObserver.read(slot,{
        afterSequence:args.afterSequence??0,
        includeImage:args.includeImage??true,
      });
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
        const current=validateNativeWindowTarget(
          await native(config.executable,'target',slot),slot);
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
        const nativeSnapshot=validateNativeReport(
          await native(config.executable,'inspect'),'inspect');
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
      keysExactly(args,['slots']);
      return summarizeGameEvidence(config.evidenceDirectory,
        args.slots ?? [1,2,3,4,5],evidence);
    }
    if (name==='plan_player_actions') {
      keysExactly(args,['actions','includeTimeline','includeClientFrames']);
      if ((args.includeTimeline!==undefined &&
           typeof args.includeTimeline!=='boolean') ||
          (args.includeClientFrames!==undefined &&
           typeof args.includeClientFrames!=='boolean') ||
          (args.includeTimeline===true && args.includeClientFrames===true)) {
        throw new Error('Invalid tool arguments');
      }
      if(args.includeClientFrames===true) {
        // Pure preview: NEVER contacts native controller or sends input.
        return compilePerClientInputFrames(config.mode,args.actions);
      }
      return args.includeTimeline===true ?
        compilePlayerInputTimeline(config.mode,args.actions) :
        planPlayerActions(config.mode,args.actions);
    }
    if (name==='stop_experimental_multi_client_actions') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['slots']);
      const slots=checkedSlots(args.slots);
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
      try {
        if(!await releaseAiSlots(slots)) {
          markAiUncertain(slots);
          throw new Error('AI_RELEASE_UNCONFIRMED');
        }
        // Only explicitly selected slots receive a new exact-owner
        // neutral ACK. Never silently clear other uncertain game slots.
        for(const slot of slots)uncertainAiSlots.delete(slot);
        return {schema:1,slots,status:'NATIVE_NEUTRAL_WRITES_ACKNOWLEDGED',
          actualGameKeyRelease:'NOT_VERIFIED',gameplayObserved:false};
      } finally {inFlight=false;}
    }
    if (name==='evaluate_experimental_multi_client_outcomes') {
      if(!multiEnabled)throw new Error('MULTI_CLIENT_EXPERIMENT_DISABLED');
      keysExactly(args,['actions','receipt','after','reviews']);
      // The same compiler defines which AI slots and outcome categories
      // were requested. A caller cannot invent an evaluation of MC01
      // human actions or an unrelated attack/interaction.
      const scheduled=prepareMultiClientPulseBatches(config.mode,args.actions);
      if(args.receipt?.actionFingerprint!==fingerprintActions(
          config.mode,args.actions)) {
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
      refuseUncertain(schedule.slots);
      if(exclusiveBusy())throw new Error('Native operation in progress; do not overlap requests');
      // A model or external caller must have an exact recent baseline for
      // EVERY targeted AI slot. Image pixels still require an explicit
      // prior read_visual_observation call; metadata isn't visual reasoning.
      const state=verifyTeamObservationBasis(
        config.mode,schedule.slots,args.basis,
        schedule.slots.map(slot=>visualObserver.read(slot,{
          includeImage:false,
        }).metadata));
      // No await between freshness check and the original controller's
      // exclusive MCP lock. The existing observer revocation callback can
      // abort exactly this observed action if a selected source disappears.
      const transportStartedAt=new Date().toISOString();
      observedSequenceSlots=[...schedule.slots];
      try {
        const receipt=await callTool('run_experimental_multi_client_actions',{
          actions:args.actions,
        });
        const transportCompletedAt=new Date().toISOString();
        const prior=state.frames.map(frame=>frame.baseline);
        return {...receipt,mode:config.mode,visualBaseline:prior,
          actionFingerprint:fingerprintActions(config.mode,args.actions),
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
        observedSequenceSlots=null;
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
      const abort=new AbortController();
      const run={abort,slots:schedule.slots};
      multiSequence=run;
      let dispatched=false,neutralConfirmed=true,transportConfirmed=false;
      try {
        dispatched=true;
        const receipt=await aiMultiBatch(config.executable,schedule.batches,
          {signal:abort.signal});
        if(receipt?.schema!==1||
           JSON.stringify(receipt?.slots)!==JSON.stringify(schedule.slots)||
           receipt?.status!=='EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED'||
           receipt?.gameplayObserved!==false) {
          throw new Error('AI_INPUT_NATIVE_RESULT_UNVERIFIED');
        }
        if(abort.signal.aborted)throw new Error('AI_SEQUENCE_ABORTED');
        transportConfirmed=true;
        return receipt;
      }finally{
        // Failed/aborted/unacknowledged input is potentially delivered.
        // A best-effort automatic ReleaseAll is not permission to replay:
        // a separate explicit stop/neutral ACK is required to recover.
        if(dispatched&&!transportConfirmed)markAiUncertain(schedule.slots);
        if(dispatched) {
          neutralConfirmed=await releaseAiSlots(schedule.slots);
          if(!neutralConfirmed)markAiUncertain(schedule.slots);
        }
        if(multiSequence===run)multiSequence=null;
        inFlight=false;
        if(!neutralConfirmed)throw new Error('AI_RELEASE_UNCONFIRMED');
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
      let dispatched=false,neutralConfirmed=true;
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
        return result;
      } finally {
        // If MCP cancels/kills the bounded native batch child, its
        // destructor cannot guarantee release. The retained controller
        // receives an independent exact-MC02 neutral-state request.
        // Failing this release is NEVER reported as sequence success.
        if(dispatched) {
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
        if(!neutralConfirmed)throw new Error('MC02_RELEASE_UNCONFIRMED');
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
          observed.push(validateNativeWindowTarget(
            await native(config.executable,'target',slot),slot));
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
        const report=await native(config.executable,'inspect');
        return planPeerSession(config.mode,validateNativeReport(report,'inspect'));
      } finally {inFlight=false;}
    }
    if (name==='agent_capabilities') {keysExactly(args,[]);return capabilities(config.mode,mc02Enabled,multiEnabled);}
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
        return validateNativeReport(
          await native(config.executable,'inspect'),'inspect');
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
        return validateNativeReport(
          await native(config.executable,operation,slot),operation);
      } finally {inFlight=false;}
    }
    throw new Error('Unknown tool');
  }
  async function handle(message) {
    if (!message || message.jsonrpc!=='2.0' || typeof message.method!=='string') return null;
    if (!Object.hasOwn(message,'id')) return null;
    const id=message.id;
    const result=(value)=>({jsonrpc:'2.0',id,result:value});
    const err=(code,message)=>({jsonrpc:'2.0',id,error:{code,message}});
    try {
      if (message.method==='initialize') {
        const declared=message.params?.protocolVersion;
        return result({protocolVersion:SUPPORTED_PROTOCOLS.has(declared)?declared:'2025-06-18',
          capabilities:{tools:{listChanged:false}},serverInfo:{name:'lazy-tester-agent',version:'0.1.0'}});
      }
      if (message.method==='ping') return result({});
      if (message.method==='tools/list') return result({tools:mc02Enabled?
        [...TOOL_LIST,EXPERIMENTAL_MC02_TOOL,...EXPERIMENTAL_MC02_SEQUENCE_TOOLS] :
        (multiEnabled?[...TOOL_LIST,...EXPERIMENTAL_MULTI_CLIENT_TOOLS,
          EXPERIMENTAL_OBSERVED_TEAM_TOOL,
          EXPERIMENTAL_TEAM_FEEDBACK_TOOL]:TOOL_LIST)});
      if (message.method==='tools/call') {
        const name=message.params?.name;
        if (typeof name!=='string') return err(-32602,'Tool name is required');
        try {
          const data=await callTool(name,message.params?.arguments??{});
          if (name==='observe_client' || name==='read_visual_observation') {
            // Cached rolling observation returns metadata alone unless a
            // newer frame is explicitly requested by the model.
            const content=[{type:'text',text:boundedMcpText(data.metadata)}];
            if (typeof data.imageBase64==='string') {
              if(data.imageBase64.length>MAX_MCP_IMAGE_CHARS) {
                throw new Error('MCP_IMAGE_RESPONSE_TOO_LARGE');
              }
              content.push({type:'image',mimeType:'image/png',
                data:data.imageBase64});
            }
            return result({content,isError:false});
          }
          return result({content:[{type:'text',text:boundedMcpText(data)}],isError:false});
        } catch(error) {
          return result({content:[{type:'text',text:publicToolError(error)}],isError:true});
        }
      }
      return err(-32601,'Method not found');
    }catch {return err(-32603,'Internal error');}
  }
  return {handle,callTool,shutdown:()=>{
    mc02SequenceAbort?.abort();
    multiSequence?.abort.abort();
    if (gpuStartup) gpuStartup.cancelled=true;
    visualObserver.shutdown();
    gpuSource?.shutdown();
  }};
}

export function serve(config) {
  const handler=createHandler(config);
  const rl=readline.createInterface({input:process.stdin,crlfDelay:Infinity,terminal:false});
  rl.once('close',()=>handler.shutdown());
  rl.on('line',async line=>{
    if (Buffer.byteLength(line)>REQUEST_LIMIT) return;
    let message;
    try{message=JSON.parse(line);}catch{return;}
    try {
      const reply=await handler.handle(message);
      if (reply) process.stdout.write(JSON.stringify(reply)+'\n');
    }catch { /* MCP stdout must remain parseable; failures are returned by handler. */ }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {serve(parseConfiguration(process.argv.slice(2)));}
  catch(error){process.stderr.write(error.message+'\n');process.exitCode=2;}
}
