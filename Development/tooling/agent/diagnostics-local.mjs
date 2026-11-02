// Local-first Minecraft Education diagnostic evidence. No Minecraft/VS Code access,
// network calls, timers, background collectors, AI requests or account/world writes.
// Explicit operator-imported normalized JSONL is retained privately and immutably.
import {createHash} from 'node:crypto';
import {lstat, mkdir, open, readdir, rename} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const MAX_BYTES=4*1024*1024;
const MAX_EVENTS=10000;
const MAX_RUNS=64;
const MAX_INDEX_BYTES=256*1024;
const MAX_MANIFEST_BYTES=4096;
const DIGEST=/^[a-f0-9]{64}$/;
const ARCHIVE_ORIGIN='EXPLICIT_LOCAL_IMPORT_NOT_GAMEPLAY_VERIFIED';
const RUN=/^[a-z][a-z0-9._-]{0,47}$/;
const CHECK=/^[a-z][a-z0-9._-]{0,63}$/;
const CODE=/^[A-Z][A-Z0-9_.-]{0,63}$/;
const TOKEN=/^[A-Z][A-Z0-9_-]{0,31}$/;
const KEYS=new Set(['schema','run','seq','tick','kind','arena',
  'check','expected','actual','code','count']);
const KINDS=new Set(['assertion','script_error','telemetry_gap',
  'state_change','metric']);

function requireId(value,re=RUN) {
  if(typeof value!=='string' || !re.test(value)) {
    throw new Error('Invalid diagnostic identifier');
  }
  return value;
}
export function validateDiagnosticsRoot(root) {
  if(typeof root!=='string'||!path.isAbsolute(root)||
     path.basename(root).toLowerCase()!=='diagnostics'||
     path.basename(path.dirname(root)).toLowerCase()!=='data') {
    throw new Error('Diagnostics store must be an absolute Data/Diagnostics directory');
  }
  return root;
}
function scalar(value) {
  return typeof value==='boolean' ||
    (typeof value==='number' && Number.isFinite(value) && Math.abs(value)<=1e9) ||
    (typeof value==='string' && TOKEN.test(value));
}
function validEvent(e,run) {
  if(!e || typeof e!=='object'||Array.isArray(e)||
     Object.keys(e).some(key=>!KEYS.has(key)) ||
     e.schema!==1 || e.run!==run || !KINDS.has(e.kind) ||
     !Number.isSafeInteger(e.seq)||e.seq<1 ||
     !Number.isSafeInteger(e.tick)||e.tick<0 ||
     (e.arena!==undefined && !CHECK.test(e.arena))) {
    throw new Error('Invalid diagnostic event schema');
  }
  if(e.kind==='assertion' || e.kind==='state_change') {
    if(!CHECK.test(e.check??'')||!scalar(e.expected)||!scalar(e.actual)) {
      throw new Error('Invalid diagnostic assertion/state');
    }
  } else if(e.kind==='script_error') {
    if(!CODE.test(e.code??'')) throw new Error('Invalid diagnostic error code');
  } else if(e.kind==='telemetry_gap') {
    if(!Number.isSafeInteger(e.count)||e.count<1||e.count>1000000) {
      throw new Error('Invalid diagnostic gap count');
    }
  } else if(e.kind==='metric') {
    if(!CHECK.test(e.check??'') || typeof e.actual!=='number' ||
       !Number.isFinite(e.actual)||Math.abs(e.actual)>1e9) {
      throw new Error('Invalid diagnostic metric');
    }
  }
  // Do not accept fields with unvalidated semantics, even though archives
  // remain private. This avoids accidental identifiers and secret payloads.
  const allowed={
    assertion:['schema','run','seq','tick','kind','arena','check','expected','actual'],
    state_change:['schema','run','seq','tick','kind','arena','check','expected','actual'],
    script_error:['schema','run','seq','tick','kind','arena','code'],
    telemetry_gap:['schema','run','seq','tick','kind','arena','count'],
    metric:['schema','run','seq','tick','kind','arena','check','actual'],
  }[e.kind];
  if(Object.keys(e).some(k=>!allowed.includes(k))) {
    throw new Error('Unexpected diagnostic event field');
  }
  return e;
}
function issueKey(kind,event) {
  return [kind,event.check??event.code??'',event.arena??''].join('|');
}
function projectEvent(e) {
  // Explicit allowlist: no raw message, filename, path, account, player,
  // arbitrary object, or screenshot is ever sent to Codex.
  return {
    seq:e.seq,tick:e.tick,kind:e.kind,
    ...(e.arena?{arena:e.arena}:{}),
    ...(e.check?{check:e.check}:{}),
    ...(e.code?{code:e.code}:{}),
    ...(e.expected!==undefined?{expected:e.expected}:{}),
    ...(e.actual!==undefined?{actual:e.actual}:{}),
    ...(e.count!==undefined?{count:e.count}:{}),
  };
}
export function analyzeDiagnosticJsonl(content,run) {
  requireId(run);
  if(typeof content!=='string'||Buffer.byteLength(content)>MAX_BYTES||
     !content.endsWith('\n')) {
    throw new Error('Diagnostic input missing final newline or above the byte budget');
  }
  const lines=content.split('\n');
  lines.pop();
  if(lines.length<1||lines.length>MAX_EVENTS) {
    throw new Error('Diagnostic event count outside allowed bounds');
  }
  const issues=new Map();
  const totals={events:0,assertions:0,matchedObservations:0,
    mismatchObservations:0,scriptErrors:0,telemetryGaps:0,
    reportedDroppedEvents:0};
  let previous=0,previousTick=0;
  function add(kind,e) {
    const key=issueKey(kind,e);
    const id=createHash('sha256').update(run+'|'+key)
      .digest('hex').slice(0,16);
    const before=issues.get(id);
    const issue=before??{id,kind,check:e.check??null,
      code:e.code??null,arena:e.arena??null,
      occurrences:0,firstSeq:e.seq,lastSeq:e.seq,samples:[]};
    issue.occurrences++;
    issue.lastSeq=e.seq;
    if(issue.samples.length>=8) issue.samples.shift();
    issue.samples.push(projectEvent(e));
    issues.set(id,issue);
  }
  for(const line of lines) {
    if(!line || Buffer.byteLength(line)>4096) {
      throw new Error('Empty or oversized diagnostic line');
    }
    let e;
    try{e=validEvent(JSON.parse(line),run);}
    catch{throw new Error('Malformed or unsupported diagnostic JSONL record');}
    if(e.seq<=previous||e.tick<previousTick) {
      throw new Error('Diagnostic sequence/tick changed or regressed; use a new run');
    }
    if(e.seq>previous+1) {
      const lost=e.seq-previous-1;
      totals.telemetryGaps++;
      totals.reportedDroppedEvents+=lost;
      add('sequence_gap',{schema:1,run,seq:e.seq,tick:e.tick,
        kind:'telemetry_gap',arena:e.arena,count:lost});
    }
    totals.events++;
    if(e.kind==='assertion') {
      totals.assertions++;
      // Equality is a deterministic observation, not game acceptance;
      // the imported file has no authenticated Minecraft-world identity.
      if(e.expected!==e.actual) {
        totals.mismatchObservations++;
        add('assertion_mismatch',e);
      } else totals.matchedObservations++;
    } else if(e.kind==='script_error') {
      totals.scriptErrors++;
      add('script_error',e);
    } else if(e.kind==='telemetry_gap') {
      totals.telemetryGaps++;
      totals.reportedDroppedEvents+=e.count;
      add('telemetry_gap',e);
    }
    previous=e.seq;previousTick=e.tick;
  }
  return {schema:1,run,origin:'EXPLICIT_LOCAL_IMPORT_NOT_GAMEPLAY_VERIFIED',
    verdict:'CANDIDATES_ONLY_NO_GAMEPLAY_ACCEPTANCE',totals,
    issueCount:issues.size,
    issues:[...issues.values()].sort((a,b)=>a.firstSeq-b.firstSeq)};
}
async function ordinaryDirectory(dir) {
  const s=await lstat(dir);
  if(!s.isDirectory()||s.isSymbolicLink()) {
    throw new Error('Diagnostics directory is not ordinary');
  }
}
async function ensureDirectory(dir) {
  try{await mkdir(dir,{mode:0o700});}
  catch(e){if(e.code!=='EEXIST')throw e;}
  await ordinaryDirectory(dir);
}
function snapshotEqual(a,b) {
  return a.dev===b.dev && a.ino===b.ino &&
    a.size===b.size && a.mtimeNs===b.mtimeNs &&
    a.ctimeNs===b.ctimeNs;
}
async function readPinned(file,maxBytes) {
  const before=await lstat(file,{bigint:true});
  if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1n||
     before.size>BigInt(maxBytes)||before.size<1n) {
    throw new Error('Diagnostic source file is not a bounded ordinary file');
  }
  const handle=await open(file,'r');
  try {
    const pinned=await handle.stat({bigint:true});
    if(!snapshotEqual(before,pinned))throw new Error('Diagnostic source changed before read');
    const size=Number(pinned.size),buffer=Buffer.alloc(size);
    let offset=0;
    while(offset<size) {
      const {bytesRead}=await handle.read(buffer,offset,size-offset,offset);
      if(bytesRead<=0)throw new Error('Incomplete diagnostic source read');
      offset+=bytesRead;
    }
    const after=await handle.stat({bigint:true});
    const named=await lstat(file,{bigint:true});
    if(!snapshotEqual(pinned,after)||!snapshotEqual(pinned,named)) {
      throw new Error('Diagnostic source changed while reading');
    }
    return buffer.toString('utf8');
  }finally{await handle.close();}
}
function runDir(root,run) {
  return path.join(validateDiagnosticsRoot(root),'runs',requireId(run));
}
// Version 2: a run is published only after a manifest is committed.
// Checksums detect unexpected changes but never authenticate Minecraft.
// Paths are fixed in source, not read from a manifest or MCP arguments.
function sha256(text) {
  return createHash('sha256').update(text,'utf8').digest('hex');
}
function digestRecord(text) {
  return {bytes:Buffer.byteLength(text,'utf8'),sha256:sha256(text)};
}
function validDigest(value,max) {
  return value && typeof value==='object'&&!Array.isArray(value)&&
    Object.keys(value).length===2&&
    Number.isSafeInteger(value.bytes)&&value.bytes>0&&value.bytes<=max&&
    typeof value.sha256==='string'&&DIGEST.test(value.sha256);
}
function createManifest(run,body,index) {
  return {schema:2,run,origin:ARCHIVE_ORIGIN,status:'COMMITTED',
    recordedAt:new Date().toISOString(),
    evidence:digestRecord(body),issueIndex:digestRecord(index),
    retention:'MANUAL_REVIEW_ONLY',minecraftGameplay:'NOT_VERIFIED'};
}
function parseManifest(text,run) {
  let m;
  try{m=JSON.parse(text);}catch{throw Error('Invalid diagnostic run manifest');}
  const keys=['schema','run','origin','status','recordedAt',
    'evidence','issueIndex','retention','minecraftGameplay'];
  if(!m||typeof m!=='object'||Array.isArray(m)||
     Object.keys(m).sort().join('|')!==keys.sort().join('|')||
     m.schema!==2||m.run!==run||m.origin!==ARCHIVE_ORIGIN||
     m.status!=='COMMITTED'||m.retention!=='MANUAL_REVIEW_ONLY'||
     m.minecraftGameplay!=='NOT_VERIFIED'||
     typeof m.recordedAt!=='string'||m.recordedAt.length!==24||
     !Number.isFinite(Date.parse(m.recordedAt))||
     !validDigest(m.evidence,MAX_BYTES)||
     !validDigest(m.issueIndex,MAX_INDEX_BYTES)) {
    throw Error('Invalid diagnostic run manifest');
  }
  return m;
}
async function writeExclusive(file,content) {
  const handle=await open(file,'wx',0o600);
  try {
    await handle.writeFile(content,'utf8');
    await handle.sync();
  }finally{await handle.close();}
}
async function storageForRun(root,run) {
  await ordinaryDirectory(path.dirname(root));
  await ordinaryDirectory(root);
  await ordinaryDirectory(path.join(root,'runs'));
  const dir=runDir(root,run);
  await ordinaryDirectory(dir);
  let content=null;
  try{content=await readPinned(path.join(dir,'manifest.json'),MAX_MANIFEST_BYTES);}
  catch(error){if(error?.code!=='ENOENT')throw error;}
  if(content!==null) {
    const manifest=parseManifest(content,run);
    await ordinaryDirectory(path.join(dir,'evidence'));
    await ordinaryDirectory(path.join(dir,'analysis'));
    return {manifest,format:'V2_COMMITTED_CHECKSUMMED',
      indexFile:path.join(dir,'analysis','issues.json'),
      evidenceFile:path.join(dir,'evidence','events.jsonl')};
  }
  // Backwards-compatible, read-only v1 route: never silently migrate.
  const indexFile=path.join(dir,'index.json');
  const evidenceFile=path.join(dir,'events.jsonl');
  try {
    const a=await lstat(indexFile),b=await lstat(evidenceFile);
    if(!a.isFile()||a.isSymbolicLink()||
       !b.isFile()||b.isSymbolicLink()) {
      throw Error('Diagnostic run is not a committed archive');
    }
  }catch(error){
    if(error?.code==='ENOENT')throw Error('Diagnostic run incomplete');
    throw error;
  }
  return {manifest:null,format:'V1_LEGACY_NO_CHECKSUM',
    indexFile,evidenceFile};
}
async function readArchiveIndex(root,run) {
  const storage=await storageForRun(root,run);
  const text=await readPinned(storage.indexFile,MAX_INDEX_BYTES);
  if(storage.manifest &&
    (Buffer.byteLength(text,'utf8')!==storage.manifest.issueIndex.bytes||
     sha256(text)!==storage.manifest.issueIndex.sha256)) {
    throw Error('Diagnostic index checksum mismatch');
  }
  let data;
  try{data=JSON.parse(text);}catch{throw Error('Invalid diagnostic index JSON');}
  const totals=['events','assertions','matchedObservations',
    'mismatchObservations','scriptErrors','telemetryGaps',
    'reportedDroppedEvents'];
  const fields=['schema','run','origin','verdict','totals','issueCount','issues'];
  if(!data||typeof data!=='object'||Array.isArray(data)||
     Object.keys(data).sort().join('|')!==fields.sort().join('|')||
     data.schema!==1||data.run!==run||data.origin!==ARCHIVE_ORIGIN||
     data.verdict!=='CANDIDATES_ONLY_NO_GAMEPLAY_ACCEPTANCE'||
     !Array.isArray(data.issues)||data.issues.length>MAX_EVENTS||
     data.issueCount!==data.issues.length||
     !data.totals||typeof data.totals!=='object'||Array.isArray(data.totals)||
     Object.keys(data.totals).length!==totals.length||
     totals.some(k=>!Number.isSafeInteger(data.totals[k])||
       data.totals[k]<0||data.totals[k]>MAX_EVENTS*1000000)) {
    throw Error('Invalid diagnostics index');
  }
  for(const item of data.issues) {
    if(!item||typeof item!=='object'||Array.isArray(item)||
       Object.keys(item).some(k=>![
         'id','kind','check','code','arena','occurrences','firstSeq','lastSeq','samples'
       ].includes(k))||
       !/^[a-f0-9]{16}$/.test(item.id??'')||
       !['assertion_mismatch','script_error','telemetry_gap','sequence_gap'].includes(item.kind)||
       (item.check!==null&&!CHECK.test(item.check))||
       (item.code!==null&&!CODE.test(item.code))||
       (item.arena!==null&&!CHECK.test(item.arena))||
       !Number.isSafeInteger(item.occurrences)||item.occurrences<1||
       !Number.isSafeInteger(item.firstSeq)||!Number.isSafeInteger(item.lastSeq)||
       item.firstSeq>item.lastSeq||!Array.isArray(item.samples)||
       item.samples.length>8) {
      throw Error('Invalid diagnostics issue index');
    }
    for(const sample of item.samples) {
      validEvent({schema:1,run,...sample},run);
    }
  }
  return {data,storage};
}
async function readArchiveEvidence(storage) {
  const text=await readPinned(storage.evidenceFile,MAX_BYTES);
  if(storage.manifest &&
    (Buffer.byteLength(text,'utf8')!==storage.manifest.evidence.bytes||
     sha256(text)!==storage.manifest.evidence.sha256)) {
    throw Error('Diagnostic evidence checksum mismatch');
  }
  return text;
}
export async function importDiagnostics({store,source,run}) {
  validateDiagnosticsRoot(store);
  requireId(run);
  if(typeof source!=='string'||!path.isAbsolute(source)||
     path.extname(source).toLowerCase()!=='.jsonl') {
    throw Error('Explicit absolute .jsonl source required');
  }
  // No auto-discovery of unverified Education Content Log paths.
  const body=await readPinned(source,MAX_BYTES);
  const index=analyzeDiagnosticJsonl(body,run);
  const serialized=JSON.stringify(index)+'\n';
  const manifestJson=JSON.stringify(createManifest(run,body,serialized))+'\n';
  if(Buffer.byteLength(serialized,'utf8')>MAX_INDEX_BYTES||
     Buffer.byteLength(manifestJson,'utf8')>MAX_MANIFEST_BYTES) {
    throw Error('Diagnostic index or manifest exceeds local budget');
  }
  await ordinaryDirectory(path.dirname(store)); // Existing protected Data only
  await ensureDirectory(store);
  const runs=path.join(store,'runs');
  await ensureDirectory(runs);
  const entries=await readdir(runs);
  if(entries.length>=MAX_RUNS)throw Error('Diagnostics local run quota reached');
  const target=runDir(store,run);
  // Reserve the destination exclusively; no overwrite, cleanup or pruning.
  // A crashed/partial import stays uncommitted and is not exposed by MCP.
  await mkdir(target,{mode:0o700});
  await mkdir(path.join(target,'evidence'),{mode:0o700});
  await mkdir(path.join(target,'analysis'),{mode:0o700});
  await writeExclusive(path.join(target,'evidence','events.jsonl'),body);
  await writeExclusive(path.join(target,'analysis','issues.json'),serialized);
  const pending=path.join(target,'manifest.pending');
  await writeExclusive(pending,manifestJson);
  // Atomic commit marker is written LAST; no v2 query opens a partial run.
  await rename(pending,path.join(target,'manifest.json'));
  return {schema:1,run,status:'LOCAL_ARCHIVED_UNVERIFIED',
    storageFormat:'V2_COMMITTED_CHECKSUMMED',totals:index.totals,
    issueCount:index.issueCount,rawDataSharedWithAI:false};
}
const missing=(root)=>({schema:1,status:root?'NO_LOCAL_DIAGNOSTICS':'NOT_CONFIGURED',
  gameplay:'NOT_VERIFIED',rawDataSharedWithAI:false});
// Fixed commit-marker inspection; never read raw evidence in a run listing.
// Incomplete imports are counted but not advertised as ready for investigation.
async function isCommittedRun(root,run) {
  const dir=runDir(root,run);
  await ordinaryDirectory(dir);
  let marker=null;
  try{marker=await lstat(path.join(dir,'manifest.json'));}
  catch(error){if(error?.code!=='ENOENT')throw error;}
  if(marker)return marker.isFile()&&!marker.isSymbolicLink();
  try {
    const index=await lstat(path.join(dir,'index.json'));
    const evidence=await lstat(path.join(dir,'events.jsonl'));
    return index.isFile()&&!index.isSymbolicLink()&&
      evidence.isFile()&&!evidence.isSymbolicLink();
  }catch(error){if(error?.code==='ENOENT')return false;throw error;}
}
export async function diagnosticsOverview(root,run) {
  if(!root)return missing(root);
  validateDiagnosticsRoot(root);
  if(run!==undefined) {
    const {data:d,storage}=await readArchiveIndex(root,requireId(run));
    return {schema:1,status:'LOCAL_UNVERIFIED',run:d.run,
      totals:d.totals,issueCount:d.issueCount,archiveFormat:storage.format,
      recordedAt:storage.manifest?.recordedAt??null,
      logicalEvidenceAndIndexBytes:storage.manifest?
        storage.manifest.evidence.bytes+storage.manifest.issueIndex.bytes:null,
      issueIndexIntegrity:storage.manifest?'SHA256_VERIFIED':'LEGACY_NOT_CHECKSUMMED',
      evidenceIntegrity:'NOT_READ',gameplay:'NOT_VERIFIED',
      rawDataSharedWithAI:false};
  }
  try {
    await ordinaryDirectory(path.dirname(root));
    await ordinaryDirectory(root);
    await ordinaryDirectory(path.join(root,'runs'));
  }catch(error){if(error?.code==='ENOENT')return missing(root);throw error;}
  const names=(await readdir(path.join(root,'runs'))).sort();
  if(names.length>MAX_RUNS)throw Error('Diagnostics run quota exceeded');
  const committed=[];
  let incompleteRunEntries=0;
  for(const name of names) {
    if(!RUN.test(name)){incompleteRunEntries++;continue;}
    try {
      if(await isCommittedRun(root,name))committed.push(name);
      else incompleteRunEntries++;
    }catch{incompleteRunEntries++;}
  }
  return {schema:1,status:'LOCAL_UNVERIFIED',
    runs:committed.slice(-10),storedRunCount:committed.length,
    incompleteRunEntries,totalRunEntries:names.length,
    gameplay:'NOT_VERIFIED',rawDataSharedWithAI:false};
}
export async function listDiagnosticIssues(root,run,{cursor=0,limit=10}={}) {
  if(!root)return missing(root);
  if(!Number.isSafeInteger(cursor)||cursor<0||
     !Number.isSafeInteger(limit)||limit<1||limit>10) {
    throw new Error('Diagnostics issue page out of bounds');
  }
  const {data:d}=await readArchiveIndex(root,requireId(run));
  const selected=d.issues.slice(cursor,cursor+limit).map(item=>({
    id:item.id,kind:item.kind,check:item.check,code:item.code,
    arena:item.arena,occurrences:item.occurrences,
    firstSeq:item.firstSeq,lastSeq:item.lastSeq,
    confidence:'UNVERIFIED_IMPORT_CANDIDATE',
  }));
  return {schema:1,status:'LOCAL_UNVERIFIED',run,
    issues:selected,total:d.issues.length,
    nextCursor:cursor+selected.length<d.issues.length?cursor+selected.length:null,
    rawDataSharedWithAI:false};
}
function lookup(d,id) {
  if(typeof id!=='string'||!/^[a-f0-9]{16}$/.test(id)) {
    throw new Error('Invalid diagnostic issue ID');
  }
  const issue=d.issues.find(x=>x.id===id);
  if(!issue)throw new Error('Diagnostic issue not found');
  return issue;
}
export async function getDiagnosticIssue(root,run,id) {
  if(!root)return missing(root);
  const {data:d}=await readArchiveIndex(root,requireId(run));
  const issue=lookup(d,id);
  return {schema:1,status:'LOCAL_UNVERIFIED',run,
    issue:{...issue,confidence:'UNVERIFIED_IMPORT_CANDIDATE',
      evidenceTool:'query_diagnostic_evidence'},
    rawDataSharedWithAI:false};
}
export async function queryDiagnosticEvidence(root,run,id,{cursor=0,limit=6}={}) {
  if(!root)return missing(root);
  if(!Number.isSafeInteger(cursor)||cursor<0||
     !Number.isSafeInteger(limit)||limit<1||limit>8) {
    throw Error('Diagnostics evidence page out of bounds');
  }
  const {data:d,storage}=await readArchiveIndex(root,requireId(run));
  const issue=lookup(d,id);
  if(issue.kind==='sequence_gap') {
    // At most eight examples retained in index; missing events have no
    // actual record to return. Do not falsely claim full event coverage.
    const records=issue.samples.slice(cursor,cursor+limit);
    return {schema:1,run,issueId:id,status:'LOCAL_UNVERIFIED',
      records,total:issue.samples.length,occurrences:issue.occurrences,
      evidenceScope:'INDEX_SAMPLES_ONLY',
      nextCursor:cursor+records.length<issue.samples.length?
        cursor+records.length:null,
      rawDataSharedWithAI:false};
  }
  const content=await readArchiveEvidence(storage);
  const records=[];
  let total=0;
  for(const line of content.trimEnd().split('\n')) {
    const event=validEvent(JSON.parse(line),run);
    if(event.kind!==issue.kind && !(issue.kind==='assertion_mismatch' &&
      event.kind==='assertion'))continue;
    if((event.check??null)!==issue.check||
       (event.code??null)!==issue.code||(event.arena??null)!==issue.arena)continue;
    if(issue.kind==='assertion_mismatch'&&event.expected===event.actual)continue;
    // Return only the requested page, with fixed memory usage for records.
    if(total>=cursor&&records.length<limit)records.push(projectEvent(event));
    total++;
  }
  return {schema:1,status:'LOCAL_UNVERIFIED',run,issueId:id,
    records,total,nextCursor:cursor+records.length<total?
      cursor+records.length:null,
    evidenceIntegrity:storage.manifest?'SHA256_VERIFIED':'LEGACY_NOT_CHECKSUMMED',
    provenance:'OPERATOR_IMPORTED_NOT_GAMEPLAY_AUTHENTICATED',
    rawDataSharedWithAI:false};
}
// An explicit CLI-only import. MCP never initiates imports or opens arbitrary
// paths; remote models cannot mutate or enumerate arbitrary user files.
if(process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const args=process.argv.slice(2);
  if(args.length!==7||args[0]!=='import'||args[1]!=='--source'||
     args[3]!=='--store'||args[5]!=='--run') {
    console.error('Usage: node diagnostics-local.mjs import --source <absolute.jsonl> --store <absolute/Data/Diagnostics> --run <id>');
    process.exitCode=2;
  } else {
    importDiagnostics({source:args[2],store:args[4],run:args[6]})
      .then(result=>console.log(JSON.stringify(result)))
      .catch(error=>{
        console.error('Diagnostics import refused: '+error.message);
        process.exitCode=1;
      });
  }
}
