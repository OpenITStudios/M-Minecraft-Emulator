// Read-only Gameplay Probe QA bridge. The team's Skills Probe remains the
// exclusive test owner; this module never sends commands or installs packs.
import {readFile,lstat} from 'node:fs/promises';
import path from 'node:path';
import {openTeamZip,sha256} from './team-update-zip.mjs';

const BASELINE=new URL('./team-source-baseline.json',import.meta.url);
const RUNNER='packs/BP/scripts/probe/runner/runner.ts';
const PROBES='packs/BP/scripts/probe/probes/';
const MAX_PROBE_FILES=100;
const MAX_AUTO_PROBES=80;
const MAX_PROBE_LINE_BYTES=4096;
const PROBE_ID=/^P[0-9]{1,3}[A-Z0-9]{0,12}$/;

function refuse() {throw new Error('PROBE_SOURCE_REVIEW_REQUIRED');}
function rootFor(entries) {
  const candidates=[...entries].filter(name=>name.endsWith(RUNNER));
  if(candidates.length!==1)refuse();
  const root=candidates[0].slice(0,-RUNNER.length);
  if(root&&!/^[A-Za-z0-9._-]{1,100}\/$/.test(root))refuse();
  return root;
}

/** Verify the actual operator ZIP, not a duplicated test catalog. */
export async function readPinnedProbeCatalog(filename) {
  if(typeof filename!=='string'||!path.isAbsolute(filename))refuse();
  const baseline=JSON.parse(await readFile(BASELINE,'utf8'));
  if(baseline.reviewState!=='APPROVED_SOURCE_REVIEW'||
     !/^[a-f0-9]{64}$/.test(baseline.provenance?.probeArchiveSha256??'')||
     !/^[a-f0-9]{64}$/.test(baseline.probeRequiredHashes?.[RUNNER]??''))refuse();
  const zip=await openTeamZip(filename);
  try {
    const root=rootFor(zip.entries);
    const digest=await zip.archiveDigest();
    if(digest!==baseline.provenance.probeArchiveSha256)refuse();
    const runner=await zip.readMember(root+RUNNER);
    if(sha256(runner)!==baseline.probeRequiredHashes[RUNNER])refuse();
    const text=runner.toString('utf8');
    // Verify the source-level gate before offering even a command preview.
    if(!text.includes('event.sourceEntity')||
       !text.includes('admins.isAdmin(player)')||
       !text.includes('arg === "auto" ? p.kind === "auto"')||
       !text.includes('"RUN_START"')||
       !text.includes('"RUN_DONE"')||
       !text.includes('"RESULT"'))refuse();

    const files=[...zip.entries].filter(name=>
      name.startsWith(root+PROBES)&&name.endsWith('.ts')&&
      !name.endsWith('.test.ts')&&!name.slice(root.length+PROBES.length).includes('/')).sort();
    if(!files.length||files.length>MAX_PROBE_FILES)refuse();
    const ids=new Set();
    let declaredAuto=0;
    let parsedAuto=0;
    for(const file of files) {
      const source=(await zip.readMember(file)).toString('utf8');
      declaredAuto += [...source.matchAll(/\bkind\s*:\s*["']auto["']/g)].length;
      // Intentional strict subset: a new dynamic/indirect auto registration
      // must be reviewed rather than silently omitted from the expected set.
      const found=[...source.matchAll(/registerProbe\s*\(\s*\{\s*id\s*:\s*["'](P[0-9]{1,3}[A-Z0-9]{0,12})["'](?:(?!registerProbe\s*\(|\brun\s*:)[\s\S]){0,600}?\bkind\s*:\s*["']auto["']/g)];
      parsedAuto+=found.length;
      for(const match of found) {
        if(!PROBE_ID.test(match[1])||ids.has(match[1]))refuse();
        ids.add(match[1]);
      }
    }
    if(parsedAuto!==declaredAuto||ids.size<1||ids.size>MAX_AUTO_PROBES)refuse();
    await zip.verifyStable();
    return Object.freeze({archiveSha256:digest,runnerSha256:sha256(runner),
      autoIds:Object.freeze([...ids].sort()),autoCount:ids.size,
      source:'OPERATOR_ZIP_PINNED_SOURCE_REVIEW',
      runtimeInstalled:false});
  } finally {await zip.close();}
}


// Approved ZIP bytes are inspected only on first use or file/baseline change.
// Normal QA needs TWO cheap filesystem metadata observations and ZERO ZIP reads,
// baseline reads, JS/TS parsing or rehashing. No watcher, daemon or extra MCP.
let cachedProbeCatalog=null;
let pendingProbeCatalog=null;
async function sourceFileStamp(file,{zip=false}={}) {
  if(zip&&(!path.isAbsolute(file)||path.extname(file).toLowerCase()!=='.zip'))refuse();
  const stat=await lstat(file,{bigint:true});
  if(!stat.isFile()||stat.isSymbolicLink()||stat.nlink!==1n||
     stat.ino===0n||stat.size<1n||
     (!zip&&stat.size>64n*1024n)||
     (zip&&stat.size<22n))refuse();
  return [stat.dev,stat.ino,stat.size,stat.mtimeNs,stat.ctimeNs]
    .map(value=>value.toString()).join(':');
}
async function sourceRevision(filename) {
  const [zipStamp,baselineStamp]=await Promise.all([
    sourceFileStamp(filename,{zip:true}),
    sourceFileStamp(BASELINE),
  ]);
  // Do not expose an operator's absolute ZIP path via MCP.
  return sha256(Buffer.from(filename+'\0'+zipStamp+'\0'+baselineStamp));
}
export async function probeSourceRevisionStatus(filename,expectedRevision) {
  if(typeof expectedRevision!=='string'||!/^[a-f0-9]{64}$/.test(expectedRevision))
    return 'UNAVAILABLE';
  try {
    return await sourceRevision(filename)===expectedRevision?'UNCHANGED':'CHANGED';
  } catch {
    return 'UNAVAILABLE';
  }
}

/** One source review per unchanged ZIP. Concurrent profiles share the review. */
export async function readPinnedProbeCatalogCached(filename) {
  const revision=await sourceRevision(filename);
  if(cachedProbeCatalog?.revision===revision) {
    return {...cachedProbeCatalog.catalog,sourceRevision:revision,
      cache:'UNCHANGED_APPROVED_ZIP'};
  }
  if(pendingProbeCatalog?.revision===revision) {
    const catalog=await pendingProbeCatalog.promise;
    return {...catalog,sourceRevision:revision,
      cache:'UNCHANGED_APPROVED_ZIP'};
  }
  // A new or updated ZIP may NOT inherit a previously approved catalog.
  cachedProbeCatalog=null;
  const inspection=(async()=>{
    const catalog=await readPinnedProbeCatalog(filename);
    if(await sourceRevision(filename)!==revision)refuse();
    cachedProbeCatalog={revision,catalog};
    return catalog;
  })();
  pendingProbeCatalog={revision,promise:inspection};
  try {
    const catalog=await inspection;
    return {...catalog,sourceRevision:revision,cache:'FRESH_SOURCE_REVIEW'};
  } finally {
    if(pendingProbeCatalog?.promise===inspection)pendingProbeCatalog=null;
  }
}

/** Compact output for AI. The Probe's result is never a map-gameplay PASS. */
export function compactProbeQaResult(report,slot) {
  const status=typeof report?.status==='string'?report.status:'UNAVAILABLE';
  const category=status==='PROBE_REPORTED_ALL_PASS'?'PROBE_REPORTED_PASS':
    status==='PROBE_REPORTED_FAILURES'?'FAIL':
    status==='PROBE_REVIEW_REQUIRED'?'NEEDS_REVIEW':
    status.startsWith('WAITING_FOR_')?'WAITING':
    status.startsWith('INCOMPLETE_')||status==='CAPTURE_FILE_UNSTABLE'?
      'INCOMPLETE':'BLOCKED';
  const counts=report?.counts;
  const safeCounts=counts&&['pass','fail','info','look'].every(
    key=>Number.isSafeInteger(counts[key])&&counts[key]>=0)?
    {pass:counts.pass,fail:counts.fail,info:counts.info,look:counts.look}:null;
  // AI needs the outcome, not repeated source/procedure prose.
  return {schema:1,slot,status:category,
    ...(['WAITING','INCOMPLETE','BLOCKED'].includes(category)?
      {reason:status}:{}),
    ...(Number.isSafeInteger(report?.expectedCount)?
      {expectedCount:report.expectedCount}:{}),
    ...(safeCounts?{counts:safeCounts}:{}),
    ...(Array.isArray(report?.missingIds)&&report.missingIds.length?
      {missingIds:report.missingIds.slice(0,8)}:{}),
    ...(typeof report?.captureIssue==='string'?
      {captureIssue:report.captureIssue}:{}),
    gameplayAcceptance:'NOT_VERIFIED'};
}

/** Only three runner marker types; no raw detail, chat, or arbitrary JSON. */
export function parseProbeCapturedLine(line) {
  if(typeof line!=='string'||Buffer.byteLength(line,'utf8')>MAX_PROBE_LINE_BYTES)return null;
  const envelope=/^(\d{4}-\d\d-\d\dT\S{1,40})\s+(?:verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if(envelope&&!Number.isFinite(Date.parse(envelope[1])))return null;
  // Skills Probe's own retained Debug Console examples use raw [probe:probe]
  // lines without timestamps. Our stable file cursor, not timestamps,
  // defines which bytes are eligible for a newly prepared QA run.
  const at=envelope?envelope[1]:null;
  let message=envelope?envelope[2]:line;
  if(envelope) {
    try {message=JSON.parse('"'+message+'"');}catch {/* ordinary log */}
  }
  const marker=/^\[probe:probe\] INFO tick=(\d{1,12}) issue=(RUN_START|RUN_DONE|RESULT) (.*)$/.exec(message);
  if(!marker)return null;
  const tick=Number(marker[1]);
  if(!Number.isSafeInteger(tick))return null;
  const suppressed=/\(suppressed ([1-9]\d{0,5})\)$/.exec(marker[3]);
  const throttled=suppressed!==null;
  if(marker[2]==='RESULT') {
    const parsed=/^id=(P[0-9]{1,3}[A-Z0-9]{0,12}) result=(pass|fail|info|look) build=(dev|release)(?:\s|$)/.exec(marker[3]);
    if(!parsed)return null;
    return {kind:'result',at,tick,id:parsed[1],result:parsed[2],
      build:parsed[3],throttled};
  }
  const parsed=/^arg=(auto|all|P[0-9]{1,3}[A-Z0-9]{0,12}) count=(\d{1,3})(?:\s|$)/.exec(marker[3]);
  if(!parsed||Number(parsed[2])>MAX_AUTO_PROBES)return null;
  return {kind:marker[2]==='RUN_START'?'start':'done',
    at,tick,arg:parsed[1],count:Number(parsed[2]),throttled};
}

const empty=(status)=>({schema:1,status,probeOutcome:'NOT_ESTABLISHED',
  sourceIdentity:'OPERATOR_NAMED_LOG_UNVERIFIED',
  gameplayAcceptance:'NOT_VERIFIED'});

/**
 * Events MUST already be read from bytes appended after the private stable
 * capture cursor. A log timestamp alone can never authenticate a new test.
 * No game-level acceptance is inferred, even from all-pass Probe output.
 */
export function reviewProbeRun(events,session,{captureStatus='CAPTURE_FILE_PRESENT',
  captureReason=null,captureTruncated=false}={}) {
  if(!session||!session.captureCursor||
     !Array.isArray(session.autoIds)||session.autoIds.length<1||
     !Array.isArray(events))return empty('NOT_PREPARED');
  if(captureStatus!=='CAPTURE_FILE_PRESENT') {
    const reason=typeof captureReason==='string'&&
      /^PROBE_[A-Z_]{2,50}$/.test(captureReason)?captureReason:null;
    return {...empty(captureStatus),source:'TEAM_PROBE_LOG',
      ...(reason?{captureIssue:reason}:{})};
  }
  const expected=new Set(session.autoIds);
  if(expected.size!==session.autoIds.length)return empty('INVALID_PROBE_CATALOG');
  if(events.some(event=>event?.throttled===true)) {
    return {...empty('INCOMPLETE_THROTTLED_MARKERS'),
      expectedCount:expected.size};
  }

  // The source logger can suppress repeated RUN_START/RUN_DONE markers.
  // Multiple or interleaved runs therefore cannot be attributed safely.
  const starts=events.map((event,index)=>event?.kind==='start'?index:-1)
    .filter(index=>index>=0);
  if(starts.length===0) {
    return {...empty(events.length?'INCOMPLETE_NO_RUN_START':'WAITING_FOR_NEW_RUN_START'),
      expectedCount:expected.size};
  }
  if(starts.length!==1||starts[0]!==0||events[0].arg!=='auto') {
    return {...empty('INCOMPLETE_UNEXPECTED_RUN'),expectedCount:expected.size};
  }
  const ends=events.map((event,index)=>event?.kind==='done'?index:-1)
    .filter(index=>index>=0);
  if(!ends.length)return {...empty('WAITING_FOR_RUN_DONE'),
    expectedCount:expected.size};
  if(ends.length!==1||ends[0]!==events.length-1||
     events[ends[0]].arg!=='auto') {
    return {...empty('INCOMPLETE_UNEXPECTED_RUN'),expectedCount:expected.size};
  }
  const start=events[0];
  const end=events.at(-1);
  if(start.tick>end.tick||events.some((event,index)=>
    !Number.isSafeInteger(event.tick)||event.tick<start.tick||
    event.tick>end.tick||
    (index>0&&event.tick<events[index-1].tick))) {
    return {...empty('INCOMPLETE_PROBE_TICK_REGRESSION'),
      expectedCount:expected.size};
  }
  const reported=events.slice(1,-1);
  const observed=new Map();
  let invalid=false;
  for(const event of reported) {
    if(event?.kind!=='result'||!expected.has(event.id)||
       observed.has(event.id)||!['pass','fail','info','look'].includes(event.result)) {
      invalid=true;
    } else {
      observed.set(event.id,event.result);
    }
  }
  const missing=session.autoIds.filter(id=>!observed.has(id));
  const countsMatch=start.count===expected.size&&end.count===expected.size&&
    reported.length===expected.size&&!invalid;
  const counts={pass:0,fail:0,info:0,look:0};
  for(const result of observed.values())counts[result]++;
  const completed=countsMatch&&missing.length===0;
  const status=!completed?'INCOMPLETE_RESULTS':
    counts.fail?'PROBE_REPORTED_FAILURES':
    counts.info||counts.look?'PROBE_REVIEW_REQUIRED':
    'PROBE_REPORTED_ALL_PASS';
  return {schema:1,status,probeOutcome:completed?
    (counts.fail?'FAIL':counts.info||counts.look?'NEEDS_REVIEW':'REPORTED_PASS'):
    'NOT_ESTABLISHED',
    expectedCount:expected.size,receivedCount:reported.length,counts,
    missingIds:missing.slice(0,24),
    from:start.at,to:end.at,
    fromTick:start.tick,toTick:end.tick,
    evidenceBoundary:'SAME_STABLE_FILE_APPENDED_BYTES_AFTER_PREPARE',
    source:'TEAM_SKILLS_PROBE_RUN_MARKERS',
    sourceIdentity:'OPERATOR_NAMED_LOG_UNVERIFIED',
    gameplayAcceptance:'NOT_VERIFIED',
    priorCaptureTruncated:captureTruncated===true,
    note:'Official Skills Probe reports only; log file identity is pinned but game/world/player origin and map gameplay are not verified.'};
}
