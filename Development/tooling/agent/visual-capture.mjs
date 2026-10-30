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
const MAX_BYTES = 6 * 1024 * 1024; // Bound the MCP image response.
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
  if (!Buffer.isBuffer(bytes) || bytes.length < 33 || bytes.length > MAX_BYTES ||
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
        stat.size<33 || stat.size>MAX_BYTES) {
      throw new Error('Cua frame file is absent, linked, or too large');
    }
    const handle=await open(screenshotPath,'r');
    let bytes;
    try {
      const current=await handle.stat();
      if (!current.isFile() || current.size<33 || current.size>MAX_BYTES) {
        throw new Error('Cua frame file changed while opening');
      }
      bytes=await handle.readFile();
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
