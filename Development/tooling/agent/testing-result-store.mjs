// Original Skills Probe terminal SUMMARY archive, never a test runner,
// telemetry collector, game input owner or gameplay acceptance authority.
// Explicit opt-in Data/Testing root. Immutable per-run exclusive directories.
import {createHash} from 'node:crypto';
import {lstat,mkdir,open,readdir,rename} from 'node:fs/promises';
import path from 'node:path';
import {MAX_AUTO_PROBES} from './probe-qa.mjs';

const ID=/^[a-f0-9]{32}$/;
const SHA=/^[a-f0-9]{64}$/;
const REASON=/^[A-Z][A-Z0-9_]{2,63}$/;
const MAX_REPORT=12*1024;
const MAX_MANIFEST=2048;
const MAX_RUNS=128;
const TERMINAL=new Set(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW',
  'INCOMPLETE','BLOCKED']);
const VERIFIED_COUNTABLE=new Set(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW']);
const sha256=value=>createHash('sha256').update(value).digest('hex');
const exact=(value,keys)=>!!value&&typeof value==='object'&&
  !Array.isArray(value)&&Object.keys(value).length===keys.length&&
  keys.every(key=>Object.hasOwn(value,key));
const validSlot=slot=>Number.isInteger(slot)&&slot>=1&&slot<=5;
const validRun=id=>typeof id==='string'&&ID.test(id);
const validDigest=value=>typeof value==='string'&&SHA.test(value);

export function validateTestingArchiveRoot(root) {
  if(typeof root!=='string'||!path.isAbsolute(root)||
     path.basename(root).toLowerCase()!=='testing'||
     path.basename(path.dirname(root)).toLowerCase()!=='data'||
     path.normalize(root)!==root)
    throw new Error('Testing archive requires an absolute Data/Testing root');
  return root;
}
async function ordinaryDirectory(dir) {
  const stat=await lstat(dir);
  if(!stat.isDirectory()||stat.isSymbolicLink())
    throw new Error('Testing archive directory is not ordinary');
}
async function ensureDirectory(dir) {
  try{await mkdir(dir,{mode:0o700});}
  catch(error){if(error?.code!=='EEXIST')throw error;}
  await ordinaryDirectory(dir);
}
async function readPinned(file,maxBytes) {
  const before=await lstat(file,{bigint:true});
  if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1n||
     before.size<1n||before.size>BigInt(maxBytes))
    throw new Error('Invalid Testing archive file');
  const handle=await open(file,'r');
  try {
    const a=await handle.stat({bigint:true});
    const same=(x,y)=>x.dev===y.dev&&x.ino===y.ino&&
      x.size===y.size&&x.mtimeNs===y.mtimeNs&&x.ctimeNs===y.ctimeNs;
    if(!same(before,a))throw new Error('Testing archive changed before read');
    const body=await handle.readFile('utf8');
    const final=await handle.stat({bigint:true});
    const named=await lstat(file,{bigint:true});
    if(!same(a,final)||!same(a,named))
      throw new Error('Testing archive changed during read');
    if(Buffer.byteLength(body,'utf8')!==Number(a.size))
      throw new Error('Truncated Testing archive');
    return body;
  } finally {await handle.close();}
}
async function writeExclusive(file,body) {
  const f=await open(file,'wx',0o600);
  try {await f.writeFile(body,'utf8');await f.sync();}
  finally {await f.close();}
}
function normalizedCounts(report) {
  if(!VERIFIED_COUNTABLE.has(report.status))return null;
  const counts=report.counts;
  if(!exact(counts,['pass','fail','info','look'])||
     !Number.isSafeInteger(report.expectedCount)||
     report.expectedCount<1||report.expectedCount>MAX_AUTO_PROBES ||
     !['pass','fail','info','look'].every(key=>
       Number.isSafeInteger(counts[key])&&counts[key]>=0&&
       counts[key]<=MAX_AUTO_PROBES))return null;
  const {pass,fail,info,look}=counts;
  if(pass+fail+info+look!==report.expectedCount ||
     report.status==='PROBE_REPORTED_PASS'&&pass!==report.expectedCount ||
     report.status==='FAIL'&&fail===0 ||
     report.status==='NEEDS_REVIEW'&&(fail!==0||info+look===0))
    return null;
  return {expected:report.expectedCount,pass,fail,info,look};
}
function terminalPayload({runId,slot,report,reportDigest}) {
  if(!validRun(runId)||!validSlot(slot)||!validDigest(reportDigest)||
     !report||typeof report!=='object'||Array.isArray(report)||
     report.terminal!==true||!TERMINAL.has(report.status)||
     report.gameplayAcceptance!=='NOT_VERIFIED'||
     (report.slot!==undefined&&report.slot!==slot)||
     (report.qaReport!==undefined&&
      (report.qaReport?.map?.status!=='NOT_TESTED'||
       report.qaReport?.acceptance!=='NOT_ACCEPTED')))
    throw new Error('Not an original source-only terminal Probe result');
  const counts=normalizedCounts(report);
  return Object.freeze({
    schema:1,kind:'ORIGINAL_SKILLS_PROBE_TERMINAL_SUMMARY',
    runId,slot,reportedOutcome:report.status,originalDigest:reportDigest,
    ...(typeof report.reason==='string'&&REASON.test(report.reason)?
      {reason:report.reason}:{}),
    ...(counts?{reportedCounts:counts}:{}),
    mapTesting:'NOT_TESTED',
    gameplayAcceptance:'NOT_VERIFIED',
    originalResult:'SOURCE_ONLY_NOT_GAME_WORLD_VERIFIED',
  });
}
function validPayload(value,runId) {
  if(!value||value.schema!==1||
     value.kind!=='ORIGINAL_SKILLS_PROBE_TERMINAL_SUMMARY'||
     value.runId!==runId||!validSlot(value.slot)||
     !TERMINAL.has(value.reportedOutcome)||
     !validDigest(value.originalDigest)||
     value.mapTesting!=='NOT_TESTED'||
     value.gameplayAcceptance!=='NOT_VERIFIED'||
     value.originalResult!=='SOURCE_ONLY_NOT_GAME_WORLD_VERIFIED')return false;
  const allowed=['schema','kind','runId','slot','reportedOutcome','originalDigest',
    ...(Object.hasOwn(value,'reason')?['reason']:[]),
    ...(Object.hasOwn(value,'reportedCounts')?['reportedCounts']:[]),
    'mapTesting','gameplayAcceptance','originalResult'];
  if(!exact(value,allowed)||
     Object.hasOwn(value,'reason')&&
       (typeof value.reason!=='string'||!REASON.test(value.reason)))
    return false;
  if(Object.hasOwn(value,'reportedCounts')){
    if(!VERIFIED_COUNTABLE.has(value.reportedOutcome))return false;
    const cc=value.reportedCounts;
    if(!exact(cc,['expected','pass','fail','info','look'])||
       !Number.isSafeInteger(cc.expected)||
       cc.expected<1||cc.expected>MAX_AUTO_PROBES ||
       !['pass','fail','info','look'].every(k=>
         Number.isSafeInteger(cc[k])&&cc[k]>=0&&cc[k]<=MAX_AUTO_PROBES)||
       cc.pass+cc.fail+cc.info+cc.look!==cc.expected ||
       value.reportedOutcome==='PROBE_REPORTED_PASS'&&cc.pass!==cc.expected||
       value.reportedOutcome==='FAIL'&&cc.fail===0||
       value.reportedOutcome==='NEEDS_REVIEW'&&
         (cc.fail!==0||cc.info+cc.look===0))return false;
  }
  return true;
}
function markerFor(payload,body) {
  return {schema:1,kind:'IMMUTABLE_TESTING_ARCHIVE_COMMIT',
    runId:payload.runId,slot:payload.slot,reportedOutcome:payload.reportedOutcome,
    originalDigest:payload.originalDigest,contentDigest:sha256(body),
    recordedAt:new Date().toISOString(),
    archiveTrust:'LOCAL_CHECKSUM_NOT_MINECRAFT_AUTHENTICATION'};
}
function validManifest(m,report,body) {
  return exact(m,['schema','kind','runId','slot','reportedOutcome',
    'originalDigest','contentDigest','recordedAt','archiveTrust'])&&
    m.schema===1&&m.kind==='IMMUTABLE_TESTING_ARCHIVE_COMMIT'&&
    m.runId===report.runId&&m.slot===report.slot&&
    m.reportedOutcome===report.reportedOutcome&&
    m.originalDigest===report.originalDigest&&
    m.contentDigest===sha256(body)&&validDigest(m.contentDigest)&&
    typeof m.recordedAt==='string'&&m.recordedAt.length<=40&&
    Number.isFinite(Date.parse(m.recordedAt))&&
    m.archiveTrust==='LOCAL_CHECKSUM_NOT_MINECRAFT_AUTHENTICATION';
}
function directory(root,runId) {
  return path.join(validateTestingArchiveRoot(root),'runs',runId);
}
function archiveView(manifest,report) {
  return {
    schema:1,status:'ARCHIVED_SOURCE_ONLY',
    runId:report.runId,slot:report.slot,reportedOutcome:report.reportedOutcome,
    ...(report.reportedCounts?{reportedCounts:report.reportedCounts}:{}),
    ...(report.reason?{reason:report.reason}:{}),
    recordedAt:manifest.recordedAt,
    originalDigest:report.originalDigest,
    archiveIntegrity:'LOCAL_SHA256_VERIFIED_NOT_AUTHENTICATED',
    mapTesting:'NOT_TESTED',gameplayAcceptance:'NOT_VERIFIED',
    nativeRunState:'NOT_VERIFIED_AFTER_ARCHIVE_READ',
  };
}
export async function readTestingArchive({store,runId}) {
  if(!validRun(runId))throw new Error('Invalid Testing archive run ID');
  const root=validateTestingArchiveRoot(store);
  await ordinaryDirectory(path.dirname(root));
  await ordinaryDirectory(root);
  await ordinaryDirectory(path.join(root,'runs'));
  const folder=directory(root,runId);
  await ordinaryDirectory(folder);
  // A missing final marker means the preceding writer crashed; never
  // reinterpret partial reports as complete or retry an older run.
  const manifestText=await readPinned(path.join(folder,'manifest.json'),MAX_MANIFEST);
  const text=await readPinned(path.join(folder,'report.json'),MAX_REPORT);
  let manifest,report;
  try{manifest=JSON.parse(manifestText);report=JSON.parse(text);}
  catch{throw new Error('Invalid Testing archive JSON');}
  if(!validPayload(report,runId)||!validManifest(manifest,report,text))
    throw new Error('Testing archive integrity mismatch');
  return archiveView(manifest,report);
}
export async function archiveTestingResult({store,runId,slot,report,reportDigest}) {
  const root=validateTestingArchiveRoot(store);
  // Never persist raw captured logs or user-provided text. The summary is
  // generated from the already-frozen original in-memory Probe report.
  const payload=terminalPayload({runId,slot,report,reportDigest});
  const reportText=JSON.stringify(payload)+'\n';
  const manifestText=JSON.stringify(markerFor(payload,reportText))+'\n';
  if(Buffer.byteLength(reportText,'utf8')>MAX_REPORT||
     Buffer.byteLength(manifestText,'utf8')>MAX_MANIFEST)
    throw new Error('Testing archive record exceeds limit');
  await ordinaryDirectory(path.dirname(root)); // Existing Data, no implicit creation
  await ensureDirectory(root);
  const runs=path.join(root,'runs');
  await ensureDirectory(runs);
  const entries=await readdir(runs);
  // Idempotent same-ID reads must remain possible after quota is full.
  if(entries.length>=MAX_RUNS&&!entries.includes(runId))
    throw new Error('Testing archive run limit reached');
  const folder=directory(root,runId);
  try{await mkdir(folder,{mode:0o700});}
  catch(error) {
    if(error?.code!=='EEXIST')throw error;
    // Exclusive retry is idempotent ONLY when the already committed
    // run has the same original digest, slot and outcome.
    const existing=await readTestingArchive({store:root,runId});
    if(existing.slot===slot&&existing.originalDigest===reportDigest&&
       existing.reportedOutcome===report.status)
      return existing;
    throw new Error('Conflicting original Testing result archive');
  }
  // No deletion or replacement after a crash. The final manifest is
  // committed last by same-directory rename, never by appending to logs.
  await writeExclusive(path.join(folder,'report.json'),reportText);
  await writeExclusive(path.join(folder,'manifest.pending'),manifestText);
  await rename(path.join(folder,'manifest.pending'),path.join(folder,'manifest.json'));
  return readTestingArchive({store:root,runId});
}


// On the FIRST coordinated request after MCP restart, an existing
// committed historical result is evidence of an earlier run that must
// not silently be replayed. This is an archive query, not live control.
export async function archivedTestingSlotReference({store,slot}) {
  if(!validSlot(slot))throw new Error('Invalid Testing archive slot');
  const root=validateTestingArchiveRoot(store);
  await ordinaryDirectory(path.dirname(root));
  try{await ordinaryDirectory(root);}
  catch(error){if(error?.code==='ENOENT')return null;throw error;}
  const runs=path.join(root,'runs');
  try{await ordinaryDirectory(runs);}
  catch(error){if(error?.code==='ENOENT')return null;throw error;}
  const names=(await readdir(runs)).sort();
  if(names.length>MAX_RUNS)throw new Error('Testing archive quota exceeded');
  let selected=null;
  for(const id of names) {
    // An unexpected entry/partial run needs inspection, not silent
    // continuation into a second Testing run on a new controller.
    if(!validRun(id))throw new Error('Unexpected Testing archive entry');
    const report=await readTestingArchive({store:root,runId:id});
    if(report.slot===slot &&
       (!selected||report.recordedAt>selected.recordedAt))
      selected={runId:id,slot,recordedAt:report.recordedAt};
  }
  return selected;
}
