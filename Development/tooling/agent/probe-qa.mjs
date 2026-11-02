// Read-only Gameplay Probe QA bridge. The team's Skills Probe remains the
// exclusive test owner; this module never sends commands or installs packs.
import {readFile} from 'node:fs/promises';
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

/** Only three runner marker types; no raw detail, chat, or arbitrary JSON. */
export function parseProbeCapturedLine(line) {
  if(typeof line!=='string'||Buffer.byteLength(line,'utf8')>MAX_PROBE_LINE_BYTES)return null;
  const envelope=/^(\d{4}-\d\d-\d\dT\S{1,40})\s+(?:verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if(!envelope||!Number.isFinite(Date.parse(envelope[1])))return null;
  let message=envelope[2];
  try {message=JSON.parse('"'+message+'"');}catch {/* ordinary log */}
  const marker=/^\[probe:probe\] INFO tick=(\d{1,12}) issue=(RUN_START|RUN_DONE|RESULT) (.*)$/.exec(message);
  if(!marker)return null;
  const tick=Number(marker[1]);
  if(!Number.isSafeInteger(tick))return null;
  const at=envelope[1];
  if(marker[2]==='RESULT') {
    const parsed=/^id=(P[0-9]{1,3}[A-Z0-9]{0,12}) result=(pass|fail|info|look) build=(dev|release)(?:\s|$)/.exec(marker[3]);
    if(!parsed)return null;
    return {kind:'result',at,tick,id:parsed[1],result:parsed[2],build:parsed[3]};
  }
  const parsed=/^arg=(auto|all|P[0-9]{1,3}[A-Z0-9]{0,12}) count=(\d{1,3})(?:\s|$)/.exec(marker[3]);
  if(!parsed||Number(parsed[2])>MAX_AUTO_PROBES)return null;
  return {kind:marker[2]==='RUN_START'?'start':'done',
    at,tick,arg:parsed[1],count:Number(parsed[2])};
}

const empty=(status)=>({schema:1,status,probeOutcome:'NOT_ESTABLISHED',
  sourceIdentity:'OPERATOR_NAMED_LOG_UNVERIFIED',
  gameplayAcceptance:'NOT_VERIFIED'});

/**
 * Matches one fresh, complete auto run only. STORED/probe:report is never
 * accepted, and a completed runner is not equivalent to gameplay success.
 */
export function reviewProbeRun(events,session,{captureStatus='CAPTURE_FILE_PRESENT',
  captureTruncated=false}={}) {
  if(!session||typeof session.preparedAt!=='string'||
     !Array.isArray(session.autoIds)||!Array.isArray(events))return empty('NOT_PREPARED');
  if(captureStatus!=='CAPTURE_FILE_PRESENT')return {
    ...empty(captureStatus),source:'TEAM_PROBE_LOG'};
  const preparedMs=Date.parse(session.preparedAt);
  if(!Number.isFinite(preparedMs))return empty('NOT_PREPARED');
  const fresh=events.filter(event=>event&&
    Number.isFinite(Date.parse(event.at))&&Date.parse(event.at)>=preparedMs);
  const first=fresh.findIndex(event=>event.kind==='start'&&event.arg==='auto');
  if(first<0)return {...empty('WAITING_FOR_NEW_RUN_START'),
    expectedCount:session.autoIds.length};
  const start=fresh[first];
  const endAt=fresh.findIndex((event,index)=>index>first&&
    event.kind==='done'&&event.arg==='auto');
  const ambiguous=fresh.slice(first+1,endAt<0?undefined:endAt)
    .some(event=>event.kind==='start');
  if(ambiguous)return {...empty('INCOMPLETE_OVERLAPPING_RUNS'),
    expectedCount:session.autoIds.length};
  if(endAt<0)return {...empty('WAITING_FOR_RUN_DONE'),
    expectedCount:session.autoIds.length};
  const end=fresh[endAt];
  const reported=fresh.slice(first+1,endAt).filter(event=>event.kind==='result');
  const expected=new Set(session.autoIds);
  const observed=new Map();
  let invalid=false;
  for(const event of reported) {
    if(!expected.has(event.id)||observed.has(event.id))invalid=true;
    else observed.set(event.id,event.result);
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
    source:'TEAM_SKILLS_PROBE_RUN_MARKERS',
    sourceIdentity:'OPERATOR_NAMED_LOG_UNVERIFIED',
    gameplayAcceptance:'NOT_VERIFIED',
    priorCaptureTruncated:captureTruncated===true,
    note:'Only the original Skills Probe result markers were counted. No game/world/player attribution or map gameplay PASS is asserted.'};
}
