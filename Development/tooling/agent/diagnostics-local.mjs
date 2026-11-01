// Local-first Minecraft Education diagnostic evidence. No Minecraft/VS Code access,
// network calls, timers, background collectors, AI requests or account/world writes.
// Explicit operator-imported normalized JSONL is retained privately and immutably.
import {createHash} from 'node:crypto';
import {lstat, mkdir, open, readFile, readdir, writeFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const MAX_BYTES=4*1024*1024;
const MAX_EVENTS=10000;
const MAX_RUNS=64;
const MAX_INDEX_BYTES=256*1024;
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
async function readIndex(root,run) {
  await ordinaryDirectory(root);
  await ordinaryDirectory(path.join(root,'runs'));
  await ordinaryDirectory(runDir(root,run));
  const data=JSON.parse(await readPinned(path.join(runDir(root,run),'index.json'),
    MAX_INDEX_BYTES));
  const totalFields=['events','assertions','matchedObservations',
    'mismatchObservations','scriptErrors','telemetryGaps',
    'reportedDroppedEvents'];
  if(data?.schema!==1 || data.run!==run ||
     data.origin!=='EXPLICIT_LOCAL_IMPORT_NOT_GAMEPLAY_VERIFIED' ||
     !Array.isArray(data.issues)||data.issues.length>MAX_EVENTS||
     data.issueCount!==data.issues.length ||
     !data.totals||typeof data.totals!=='object'||
     Object.keys(data.totals).length!==totalFields.length ||
     totalFields.some(field=>!Number.isSafeInteger(data.totals[field])||
       data.totals[field]<0||data.totals[field]>MAX_EVENTS*1000000)) {
    throw new Error('Invalid diagnostics index');
  }
  // Re-derive safe issue projections from allowlisted event samples. A locally
  // tampered index must not smuggle arbitrary strings into an MCP response.
  for(const item of data.issues) {
    if(!item || Object.keys(item).some(key=>![
      'id','kind','check','code','arena','occurrences','firstSeq','lastSeq','samples'
    ].includes(key)) || !/^[a-f0-9]{16}$/.test(item.id??'') ||
       !['assertion_mismatch','script_error','telemetry_gap','sequence_gap'].includes(item.kind) ||
       (item.check!==null&&!CHECK.test(item.check)) ||
       (item.code!==null&&!CODE.test(item.code)) ||
       (item.arena!==null&&!CHECK.test(item.arena)) ||
       !Number.isSafeInteger(item.occurrences)||item.occurrences<1 ||
       !Number.isSafeInteger(item.firstSeq)||!Number.isSafeInteger(item.lastSeq) ||
       !Array.isArray(item.samples)||item.samples.length>8) {
      throw new Error('Invalid diagnostics issue index');
    }
    for(const sample of item.samples) {
      validEvent({schema:1,run,...sample},run);
    }
  }
  return data;
}
export async function importDiagnostics({store,source,run}) {
  validateDiagnosticsRoot(store);
  requireId(run);
  if(typeof source!=='string'||!path.isAbsolute(source)||
     path.extname(source).toLowerCase()!=='.jsonl') {
    throw new Error('Explicit absolute .jsonl source required');
  }
  // No auto-discovery of Minecraft Education content log paths or execution.
  const body=await readPinned(source,MAX_BYTES);
  const index=analyzeDiagnosticJsonl(body,run);
  const serialized=JSON.stringify(index)+'\n';
  if(Buffer.byteLength(serialized,'utf8')>MAX_INDEX_BYTES) {
    throw new Error('Diagnostic index exceeds local byte budget');
  }
  await ordinaryDirectory(path.dirname(store)); // existing protected Data only
  await ensureDirectory(store);
  const runs=path.join(store,'runs');
  await ensureDirectory(runs);
  const entries=await readdir(runs);
  if(entries.length>=MAX_RUNS)throw new Error('Diagnostics local run quota reached');
  const target=runDir(store,run);
  // Exclusive create; no overwrite, prune, or cleanup of any existing data.
  await mkdir(target,{mode:0o700});
  await writeFile(path.join(target,'events.jsonl'),body,
    {flag:'wx',mode:0o600});
  await writeFile(path.join(target,'index.json'),
    serialized,{flag:'wx',mode:0o600});
  return {schema:1,run,status:'LOCAL_ARCHIVED_UNVERIFIED',
    totals:index.totals,issueCount:index.issueCount,
    rawDataSharedWithAI:false};
}
const missing=(root)=>({schema:1,status:root?'NO_LOCAL_DIAGNOSTICS':'NOT_CONFIGURED',
  gameplay:'NOT_VERIFIED',rawDataSharedWithAI:false});
export async function diagnosticsOverview(root,run) {
  if(!root)return missing(root);
  validateDiagnosticsRoot(root);
  if(run!==undefined) {
    const d=await readIndex(root,requireId(run));
    return {schema:1,status:'LOCAL_UNVERIFIED',run:d.run,
      totals:d.totals,issueCount:d.issueCount,gameplay:'NOT_VERIFIED',
      rawDataSharedWithAI:false};
  }
  try {
    await ordinaryDirectory(root);
    await ordinaryDirectory(path.join(root,'runs'));
  }catch(e){if(e.code==='ENOENT')return missing(root);throw e;}
  const names=(await readdir(path.join(root,'runs'))).sort();
  if(names.length>MAX_RUNS)throw new Error('Diagnostics run quota exceeded');
  return {schema:1,status:'LOCAL_UNVERIFIED',
    runs:names.filter(name=>RUN.test(name)).slice(-10),
    totalRunEntries:names.length,gameplay:'NOT_VERIFIED',
    rawDataSharedWithAI:false};
}
export async function listDiagnosticIssues(root,run,{cursor=0,limit=10}={}) {
  if(!root)return missing(root);
  if(!Number.isSafeInteger(cursor)||cursor<0||
     !Number.isSafeInteger(limit)||limit<1||limit>10) {
    throw new Error('Diagnostics issue page out of bounds');
  }
  const d=await readIndex(root,requireId(run));
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
  const d=await readIndex(root,requireId(run));
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
    throw new Error('Diagnostics evidence page out of bounds');
  }
  const d=await readIndex(root,requireId(run));
  const issue=lookup(d,id);
  if(issue.kind==='sequence_gap') {
    return {schema:1,run,issueId:id,status:'LOCAL_UNVERIFIED',
      records:issue.samples,total:issue.occurrences,nextCursor:null,
      rawDataSharedWithAI:false};
  }
  const content=await readPinned(path.join(runDir(root,run),'events.jsonl'),MAX_BYTES);
  const matches=[];
  for(const line of content.trimEnd().split('\n')) {
    const e=validEvent(JSON.parse(line),run);
    if(e.kind!==issue.kind && !(issue.kind==='assertion_mismatch' && e.kind==='assertion'))continue;
    if((e.check??null)!==issue.check || (e.code??null)!==issue.code ||
       (e.arena??null)!==issue.arena)continue;
    if(issue.kind==='assertion_mismatch' && e.expected===e.actual)continue;
    matches.push(projectEvent(e));
  }
  const records=matches.slice(cursor,cursor+limit);
  return {schema:1,status:'LOCAL_UNVERIFIED',run,issueId:id,
    records,total:matches.length,
    nextCursor:cursor+records.length<matches.length?cursor+records.length:null,
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
