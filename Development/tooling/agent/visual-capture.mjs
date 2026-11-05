// One-shot, read-only Cua Driver window capture. No global desktop capture,
// keyboard/mouse actions, daemon launch, account access or game injection.
// Source authored only: Windows/Cua/Minecraft acceptance NOT_EXECUTED.
import {execFile} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstat, mkdtemp, open, rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {promisify} from 'node:util';

const execFileAsync = promisify(execFile);
const PNG_SIGNATURE = Buffer.from([137,80,78,71,13,10,26,10]);
export const MAX_PNG_BYTES = 3 * 1024 * 1024; // Fits the observer's 4 MiB base64 budget.
const MAX_DIMENSION = 1536;
const REQUESTED_LONG_EDGE = 1280;
const CAPTURE_TIMEOUT_MS = 20_000;

export function validateCuaExecutable(value) {
  if (typeof value!=='string' || !path.isAbsolute(value) ||
      path.basename(value).toLowerCase()!=='cua-driver.exe') {
    throw new TypeError('Cua Driver requires an absolute cua-driver.exe path');
  }
  return value;
}

export function inspectPng(bytes) {
  if (!Buffer.isBuffer(bytes) || bytes.length < 33 || bytes.length > MAX_PNG_BYTES ||
      !bytes.subarray(0,8).equals(PNG_SIGNATURE) ||
      bytes.readUInt32BE(8)!==13 ||
      bytes.toString('ascii',12,16)!=='IHDR') {
    throw new Error('Cua returned no valid bounded PNG frame');
  }
  const width=bytes.readUInt32BE(16),height=bytes.readUInt32BE(20);
  if (width<1 || height<1 || width>MAX_DIMENSION || height>MAX_DIMENSION) {
    throw new Error('Cua frame dimensions exceed capture limits');
  }
  return {width,height,bytes:bytes.length,
    sha256:createHash('sha256').update(bytes).digest('hex')};
}

// Read an already-existing screenshot with an explicit 3 MiB allocation
// ceiling. FileHandle.readFile() would not make the size fixed by OUR
// admission contract if the captured file changed after its initial stat.
// Check the same opened file again after full positional reads.
export async function readBoundedPng(handle) {
  const before=await handle.stat({bigint:true});
  if (!before.isFile() || before.size<33n ||
      before.size>BigInt(MAX_PNG_BYTES)) {
    throw new Error('Cua frame changed or exceeds the PNG budget');
  }
  const length=Number(before.size);
  const bytes=Buffer.allocUnsafe(length);
  let position=0;
  while(position<length) {
    const {bytesRead}=await handle.read(bytes,position,
      length-position,position);
    if (!Number.isSafeInteger(bytesRead) || bytesRead<1 ||
        bytesRead>length-position) {
      throw new Error('Cua frame incomplete during bounded read');
    }
    position+=bytesRead;
  }
  const after=await handle.stat({bigint:true});
  if (!after.isFile() || after.dev!==before.dev ||
      after.ino!==before.ino || after.size!==before.size ||
      after.mtimeNs!==before.mtimeNs ||
      after.ctimeNs!==before.ctimeNs) {
    throw new Error('Cua frame changed during bounded read');
  }
  return bytes;
}

// Source-only semantic inspection of ONE already-verified native window.
// Never takes a screenshot, controls Windows input, or returns provider raw
// value text, window titles, screenshot data or action-ready element tokens.
const CUA_UIA_MAX_ELEMENTS=64;
const CUA_UIA_MAX_DEPTH=10;
const CUA_UIA_MAX_STDOUT_BYTES=64*1024;
const CUA_UIA_LABEL_ROLES=new Set([
  'Button','MenuItem','TabItem','CheckBox','RadioButton','Hyperlink',
]);
const CUA_UIA_ACTIONS=new Set([
  'invoke','toggle','select','expand','collapse','set_value',
]);
export function buildCuaSemanticWindowQuery(target) {
  // An unrelated window name is never sufficient authority; the caller
  // pins the original controller's PID/HWND/birth/session before and after.
  if (!target || !Number.isSafeInteger(target.pid) || target.pid<1 ||
      !Number.isSafeInteger(target.windowId) || target.windowId<1) {
    throw new TypeError('Native verified PID and HWND are required');
  }
  return {pid:target.pid,window_id:target.windowId,
    include_accessibility_tree:true,include_screenshot:false,
    tree_format:'elements',max_elements:CUA_UIA_MAX_ELEMENTS,
    max_depth:CUA_UIA_MAX_DEPTH,timeout_ms:3000};
}
export function projectCuaSemanticWindow(result,target) {
  const fail=()=>{throw new Error('CUA_UIA_READ_UNAVAILABLE');};
  if (!result || Array.isArray(result) || typeof result!=='object' ||
      result.pid!==target?.pid || result.window_id!==target?.windowId ||
      !Array.isArray(result.elements) ||
      result.elements.length>CUA_UIA_MAX_ELEMENTS ||
      typeof result.elements_complete!=='boolean' ||
      (result.degraded!==undefined && typeof result.degraded!=='boolean') ||
      (result.truncated!==undefined && typeof result.truncated!=='boolean') ||
      result.screenshot_png_b64!==undefined ||
      result.screenshot_file_path!==undefined ||
      result.screenshot_mime_type!==undefined) fail();
  const elements=result.elements.map(row=>{
    if (!row || Array.isArray(row) || typeof row!=='object' ||
        !Number.isInteger(row.element_index) || row.element_index<0 ||
        row.element_index>5000 ||
        typeof row.role!=='string' || row.role.length>64 ||
        (row.actions!==undefined &&
          (!Array.isArray(row.actions) || row.actions.length>12 ||
           row.actions.some(a=>typeof a!=='string' || a.length>40)))) fail();
    const role=row.role.replace(/[^a-zA-Z0-9 ]/g,'').slice(0,40);
    const actions=(row.actions??[]).filter(a=>CUA_UIA_ACTIONS.has(a));
    // Cua may fall back to a field's *value* for its label; never
    // expose an edit/document/value contents as the AI's button label.
    const label=CUA_UIA_LABEL_ROLES.has(role) &&
      typeof row.label==='string' && row.label!==row.value ?
      row.label.replace(/[\x00-\x1f\x7f\u202a-\u202e]/g,' ')
        .trim().slice(0,80):null;
    return {index:row.element_index,role,label,
      enabled:typeof row.enabled==='boolean'?row.enabled:null,
      actions};
  });
  // Exact provider token/index consistency at observation time only.
  // Another Cua daemon client may invalidate the snapshot immediately.
  // Never return the snapshot id or element tokens to MCP clients.
  const indices=new Set();
  const snapshotProvenance=
    typeof result.snapshot_id==='string' &&
    /^s[0-9a-f]{8}$/.test(result.snapshot_id) &&
    result.elements.every(row=>{
      if(indices.has(row.element_index))return false;
      indices.add(row.element_index);
      return row.element_token===`${result.snapshot_id}:${row.element_index}`;
    }) ? 'TOKEN_INDEX_MATCH_AT_READ' : 'UNVERIFIED';
  const partial=result.elements_complete!==true ||
    result.truncated===true || result.degraded===true;
  return {schema:1,provider:'CUA_WINDOWS_UIA_READ_ONLY',
    status:elements.length===0?'UIA_EMPTY_OR_UNAVAILABLE':
      partial?'UIA_PARTIAL_UNVERIFIED':'UIA_ELEMENTS_READ_UNVERIFIED',
    elementsComplete:!partial,
    snapshotProvenance,elements,
    screenshotCaptured:false,actionPermitted:false,
    gameplayInput:'NOT_VERIFIED'};
}
// P59-P60: read-only candidacy, never a grant. A matching token may
// already be stale; Cua click can fall back to PostMessage/injection,
// and some UIA handlers can self-activate. No execution is authorized.
export function assessCuaStrictBackgroundAction(inspection,{
  elementIndex,action,
}={}) {
  if(!Number.isSafeInteger(elementIndex) || elementIndex<0 ||
     elementIndex>5000 || action!=='invoke') {
    throw new TypeError('Only an indexed UIA invoke assessment is supported');
  }
  const result={
    schema:1,deliveryMode:'STRICT_BACKGROUND',status:'BACKGROUND_UNAVAILABLE',
    reason:null,elementIndex,action,
    actionDispatched:false,actionPermitted:false,
    foregroundFallback:false,windowActivated:false,
    screenshotCaptured:false,backgroundEffect:'NOT_VERIFIED',
    backgroundRoute:'BLOCKED_NOT_INVOKE_ONLY',
    targetPostcondition:'NOT_DECLARED',focusIsolation:'NOT_VERIFIED',
    tokenExported:false,gameInput:'NOT_VERIFIED',
  };
  const refuse=reason=>({...result,reason});
  if(inspection?.status==='BACKGROUND_UNAVAILABLE') {
    return refuse('CUA_UIA_NOT_CONFIGURED');
  }
  if(inspection?.schema!==1 ||
     inspection?.provider!=='CUA_WINDOWS_UIA_READ_ONLY' ||
     inspection?.screenshotCaptured!==false ||
     inspection?.actionPermitted!==false ||
     !Array.isArray(inspection?.elements) ||
     inspection.elements.length>CUA_UIA_MAX_ELEMENTS) {
    return refuse('UNTRUSTED_UIA_INSPECTION');
  }
  const identity=inspection.identity;
  if(!Number.isInteger(inspection.slot) ||
     inspection.slot<1 || inspection.slot>5 ||
     !identity || !Number.isSafeInteger(identity.pid) || identity.pid<1 ||
     !Number.isSafeInteger(identity.windowId) || identity.windowId<1 ||
     !Number.isSafeInteger(identity.sessionId) || identity.sessionId<0 ||
     typeof identity.createdFileTime!=='string' ||
     !/^\d{1,20}$/.test(identity.createdFileTime)) {
    return refuse('NATIVE_TARGET_UNVERIFIED');
  }
  if(inspection.status!=='UIA_ELEMENTS_READ_UNVERIFIED' ||
     inspection.elementsComplete!==true) {
    return refuse('UIA_INCOMPLETE_OR_DEGRADED');
  }
  if(inspection.snapshotProvenance!=='TOKEN_INDEX_MATCH_AT_READ') {
    return refuse('UIA_SNAPSHOT_TOKEN_PROVENANCE_UNVERIFIED');
  }
  // A malformed or repeated source index must never become authority for
  // whichever element happens to be first in an untrusted projection.
  const matches=inspection.elements.filter(e=>e?.index===elementIndex);
  if(matches.length!==1) return refuse('UIA_ELEMENT_ABSENT_OR_AMBIGUOUS');
  const element=matches[0];
  if(element.role!=='Button' || element.enabled!==true ||
     typeof element.label!=='string' || !element.label.trim() ||
     !Array.isArray(element.actions) ||
     !element.actions.includes('invoke')) {
    return refuse('UIA_INVOKE_NOT_ADVERTISED_ON_ENABLED_BUTTON');
  }
  return {...result,status:'SEMANTIC_CANDIDATE_NOT_EXECUTED',
    reason:'UIA_INVOKE_ADVERTISED_BACKGROUND_UNPROVEN',
    target:{slot:inspection.slot,pid:identity.pid,
      windowId:identity.windowId,sessionId:identity.sessionId,
      createdFileTime:identity.createdFileTime},
    element:{index:elementIndex,role:'Button',label:element.label},
    next:'REQUIRE_INVOKE_ONLY_ROUTE_NO_FOCUS_PROOF_AND_TARGET_POSTCONDITION'};
}

export async function inspectCuaWindowElements(cuaExecutable,target,{
  platform=process.platform,statFile=lstat,execute=execFileAsync,
}={}) {
  validateCuaExecutable(cuaExecutable);
  if(platform!=='win32')throw new Error('CUA_UIA_WINDOWS_ONLY');
  const query=buildCuaSemanticWindowQuery(target);
  // Same existing daemon-backed, allowlisted Cua Driver executable. Never
  // auto-start a daemon or use an arbitrary command/target supplied by Codex.
  const imageStat=await statFile(cuaExecutable);
  if(!imageStat.isFile() || imageStat.isSymbolicLink()) {
    throw new Error('CUA_UIA_READ_UNAVAILABLE');
  }
  let output;
  try {
    output=await execute(cuaExecutable,
      ['call','get_window_state',JSON.stringify(query)],
      {shell:false,windowsHide:true,timeout:7000,
        maxBuffer:CUA_UIA_MAX_STDOUT_BYTES});
  } catch {throw new Error('CUA_UIA_READ_UNAVAILABLE');}
  let structured;
  try {structured=JSON.parse(output.stdout.trim());}
  catch {throw new Error('CUA_UIA_READ_UNAVAILABLE');}
  return projectCuaSemanticWindow(structured,target);
}

export function buildCuaWindowQuery(target) {
  if (!target || !Number.isSafeInteger(target.pid) || target.pid<1 ||
      !Number.isSafeInteger(target.windowId) || target.windowId<1) {
    throw new TypeError('Native verified PID and HWND are required');
  }
  // display_only is the documented visual-preview route. It prevents a
  // frame sampler from replacing the interactive Cua action snapshot.
  return {
    pid:target.pid,window_id:target.windowId,
    include_accessibility_tree:false,include_screenshot:true,
    display_only:true,screenshot_format:'png',
    max_image_dimension:REQUESTED_LONG_EDGE,
  };
}

export async function captureCuaWindow(cuaExecutable,target) {
  validateCuaExecutable(cuaExecutable);
  if (process.platform!=='win32') throw new Error('Cua Windows capture requires Windows');
  const query=buildCuaWindowQuery(target);
  const imageStat=await lstat(cuaExecutable);
  if (!imageStat.isFile() || imageStat.isSymbolicLink()) {
    throw new Error('Cua Driver executable must be an ordinary local file');
  }
  // Random private temporary output; not written to Git, game profile, world,
  // or caller-chosen path. The screenshot is deleted before returning.
  const temporary=await mkdtemp(path.join(tmpdir(),'lazy-tester-observe-'));
  const screenshotPath=path.join(temporary,'window.png');
  try {
    // Query is window-specific and does not change the Cua action snapshot.
    // This is the read-only Cua CLI tool (NOT a general command runner).
    // The separate Cua daemon must already be running; never auto-start it.
    try {
      await execFileAsync(cuaExecutable,
        ['call','get_window_state',JSON.stringify(query),
          '--screenshot-out-file',screenshotPath],
        {shell:false,windowsHide:true,timeout:CAPTURE_TIMEOUT_MS,
          maxBuffer:64*1024});
    } catch {
      throw new Error('Cua window capture unavailable or refused; no screenshot accepted');
    }
    const stat=await lstat(screenshotPath);
    if (!stat.isFile() || stat.isSymbolicLink() ||
        stat.size<33 || stat.size>MAX_PNG_BYTES) {
      throw new Error('Cua frame file is absent, linked, or too large');
    }
    const handle=await open(screenshotPath,'r');
    let bytes;
    try {
      bytes=await readBoundedPng(handle);
    } finally {await handle.close();}
    const details=inspectPng(bytes);
    return {
      ...details,mimeType:'image/png',imageBase64:bytes.toString('base64'),
      provider:'CUA_DRIVER_WINDOW_CAPTURE',
      frameFreshness:'UNVERIFIED_SINGLE_FRAME',
    };
  } finally {
    // Failure to erase a captured image is an error, not silent retention.
    await rm(temporary,{recursive:true,force:true});
  }
}
