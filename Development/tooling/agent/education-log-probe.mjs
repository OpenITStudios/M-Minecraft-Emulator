// Minecraft Education Script API Content Log compatibility probe.
// SOURCE ONLY: no game, network, debugger, background observer, or automatic
// Minecraft path lookup. Explicit local capture files are never modified.
// This examines only a controlled test marker; not ordinary user log data.
import {createHash} from 'node:crypto';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {readPinned} from './diagnostics-local.mjs';

const MAX_BYTES=4*1024*1024;
const MAX_LINES=12000;
const MAX_LINE_BYTES=4096;
const MAX_MARKER_BYTES=512;
const RUN=/^[a-z][a-z0-9._-]{0,47}$/;
const TAG='[LT_EDU_PROBE]';
const MODES=new Set(['marker-lines','timestamped-print']);

function safeRun(run) {
  if(typeof run!=='string'||!RUN.test(run)) {
    throw new TypeError('Invalid nonsecret probe run ID');
  }
  return run;
}

function extractMessage(line,format) {
  if(format==='marker-lines')return line;
  // This is ONLY a potential debug capture wrapper, not a claim about
  // Minecraft Education's actual Content Log file layout.
  const match=/^(\d{4}-\d\d-\d\dT\S{1,32})\s+(verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if(!match||!Number.isFinite(Date.parse(match[1])))return null;
  let message=match[3];
  if(message.startsWith('"')&&message.endsWith('"')) {
    try{message=JSON.parse(message);}catch{return null;}
  }else if(message.startsWith(TAG)&&message.includes('\\\"')) {
    // Some captured PrintEvent lines retain JSON-string-escaped quotes.
    try{message=JSON.parse('"'+message+'"');}catch{return null;}
  }
  return typeof message==='string'?message:null;
}

function recognize(message,run) {
  if(typeof message!=='string'||!message.startsWith(TAG))return null;
  const payload=message.slice(TAG.length);
  if(Buffer.byteLength(payload,'utf8')>MAX_MARKER_BYTES) {
    return {valid:false,reason:'OVERSIZED_MARKER'};
  }
  let item;
  try{item=JSON.parse(payload);}catch{
    return {valid:false,reason:'INVALID_MARKER_JSON'};
  }
  const allowed=['schema','run','seq','tick','kind','phase'];
  if(!item||typeof item!=='object'||Array.isArray(item)||
     Object.keys(item).sort().join('|')!==allowed.sort().join('|')||
     item.schema!==1||item.run!==run||item.kind!=='script_probe'||
     !Number.isSafeInteger(item.tick)||item.tick<0||
     (item.seq!==1&&item.seq!==2)||
     item.phase!==(item.seq===1?'START':'DEFERRED')) {
    return {valid:false,reason:'INVALID_MARKER_CONTRACT'};
  }
  return {valid:true,seq:item.seq,tick:item.tick,phase:item.phase};
}

export function summarizeEducationCapture(text,{run,format}={}) {
  safeRun(run);
  if(!MODES.has(format))throw new TypeError('Unknown explicit capture format');
  if(typeof text!=='string'||Buffer.byteLength(text,'utf8')>MAX_BYTES||
     (text.length>0&&!text.endsWith('\n'))) {
    throw new TypeError('Capture exceeds budget or ends in partial line');
  }
  const rawLines=text.length===0?[]:text.split('\n').slice(0,-1);
  if(rawLines.length>MAX_LINES)throw new TypeError('Too many capture lines');
  const found=[];
  let markerCandidates=0,invalidMarkers=0,overlongLines=0,notRecognized=0;
  for(const raw of rawLines) {
    const line=raw.endsWith('\r')?raw.slice(0,-1):raw;
    if(Buffer.byteLength(line,'utf8')>MAX_LINE_BYTES) {
      overlongLines++;
      continue;
    }
    const msg=extractMessage(line,format);
    if(msg===null) {notRecognized++;continue;}
    const candidate=recognize(msg,run);
    if(!candidate) {notRecognized++;continue;}
    markerCandidates++;
    if(!candidate.valid) {invalidMarkers++;continue;}
    found.push(candidate);
  }
  const first=found.filter(x=>x.seq===1),last=found.filter(x=>x.seq===2);
  const complete=invalidMarkers===0&&overlongLines===0&&
    found.length===2&&first.length===1&&last.length===1&&
    found[0].seq===1&&found[1].seq===2&&
    last[0].tick>first[0].tick;
  const status=overlongLines>0?'UNRELIABLE_CAPTURE_OVERSIZED_LINES':
    complete?'CANDIDATE_MARKER_PAIR_FOUND':
    found.length>0?'PARTIAL_OR_REPEATED_MARKERS':
    markerCandidates>0?'REJECTED_MARKERS':'NO_RECOGNIZED_MARKER';
  return {
    schema:1,source:'OPERATOR_SUPPLIED_FILE_NOT_MINECRAFT_AUTHENTICATED',
    run,format,status,linesInspected:rawLines.length,
    markerCandidates,validMarkerCount:found.length,
    invalidMarkers,overlongLines,unrecognizedLines:notRecognized,
    candidateTickProgress:complete?last[0].tick-first[0].tick:null,
    minecraftEducationTransport:'NOT_VERIFIED',
    worldIdentity:'NOT_VERIFIED',
    gameplay:'NOT_VERIFIED',rawLogSharedWithAI:false,
    next:'COMPARE_WITH_REAL_EDUCATION_CAPTURE_AFTER_SEPARATE_AUTHORIZATION',
  };
}

// Optional operator-only CLI: no MCP tools, no automatic collection.
// Output contains only fixed counts/status/hash and safe supplied run ID;
// never print source paths, player names or arbitrary log contents.
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  const args=process.argv.slice(2);
  if(args.length!==7||args[0]!=='inspect'||args[1]!=='--source'||
     args[3]!=='--run'||args[5]!=='--format') {
    console.error('Usage: education-log-probe.mjs inspect --source <absolute .log/.txt> --run <safe-id> --format marker-lines|timestamped-print');
    process.exitCode=2;
  }else{
    const source=args[2],run=args[4],format=args[6];
    if(!path.isAbsolute(source)||!['.log','.txt'].includes(path.extname(source).toLowerCase())) {
      console.error('Capture probe refused: invalid explicit capture source');
      process.exitCode=2;
    }else {
      readPinned(source,MAX_BYTES).then(text=>{
        const result=summarizeEducationCapture(text,{run,format});
        const sha256=createHash('sha256').update(text,'utf8').digest('hex');
        console.log(JSON.stringify({...result,sourceSha256:sha256}));
      }).catch(()=>{
        console.error('Capture probe refused: source unavailable or unsupported');
        process.exitCode=1;
      });
    }
  }
}
