// Bounded, identity-pinned capture read. No telemetry producer or state store.
import {createHash} from 'node:crypto';
import {lstat,open} from 'node:fs/promises';
import path from 'node:path';
import {slotOf} from './agent-contract.mjs';
import {parseProbeCapturedLine} from './probe-qa.mjs';

const EVIDENCE_TAIL_BYTES = 256 * 1024;
export const EVIDENCE_LIMIT = 20;
const TEAM_CAPTURE_MAX_LINE_BYTES = 24 * 1024; // team analytics/results contract

export function parseCapturedLine(line) {
  const match=/^(\d{4}-\d\d-\d\dT\S+)\s+(verbose|log|warn|error|stop)\s+([\s\S]+)$/.exec(line);
  if(!match||match[1].length>45||
     !Number.isFinite(Date.parse(match[1])) ||
     Buffer.byteLength(match[3],'utf8')>TEAM_CAPTURE_MAX_LINE_BYTES*2) {
    return null;
  }
  // The team's direct debugger capture uses JSON-string-escaped PrintEvent
  // text without the enclosing quotes; keep bounded raw legacy lines too.
  let message=match[3];
  try{message=JSON.parse('"'+message+'"');}
  catch{/* The existing capture may already contain unescaped text. */}
  // A marker embedded in ordinary chat/console text is NOT accepted.
  const tagged=/^\s*(\[ANALYTICS\]|\[GAME_RESULTS\])\s*(\{[\s\S]*)$/.exec(message);
  if(!tagged)return null;
  if(Buffer.byteLength(tagged[1]+' '+tagged[2],'utf8')>
     TEAM_CAPTURE_MAX_LINE_BYTES)return null;
  let data;
  try{data=JSON.parse(tagged[2]);}catch{return null;}
  if(!data||typeof data!=='object'||Array.isArray(data))return null;
  const analytics=tagged[1]==='[ANALYTICS]';
  const kind=analytics?data.type:data.record_type;
  if(typeof kind!=='string'||!kind.trim()||kind.length>80||
     (data.schema!==undefined&&data.schema!==1))return null;
  if(!analytics) {
    // Team-owned result records include a final export_summary carrying
    // completeness/failure counts. It is not a gameplay_result/PASS.
    if(data.schema!==1||
       !['gameplay_result','level_score','export_summary'].includes(kind))return null;
    if(kind==='export_summary' &&
       (!data.counts||typeof data.counts!=='object'||
        Array.isArray(data.counts)||
        Object.keys(data.counts).length>64||
        Object.entries(data.counts).some(([key,value])=>
          key.length>64||!Number.isSafeInteger(value)||value<0||
          value>1000000)))return null;
  }
  return {observedAt:match[1],level:match[2],
    type:analytics?'analytics':'game_results',data};
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

// Shared stable-file cursor validation. This attests file continuity only,
// not the Minecraft world, player, or authenticity of appended records.
export function validCaptureCursor(cp) {
  return !!cp&&cp.kind==='STABLE_CAPTURE_CURSOR_V1'&&
    /^[a-f0-9]{64}$/.test(cp.fileIdentity??'')&&
    Number.isSafeInteger(cp.byteOffset)&&cp.byteOffset>=0&&
    Number.isInteger(cp.anchorBytes)&&cp.anchorBytes>=0&&
    cp.anchorBytes<=128&&cp.anchorBytes<=cp.byteOffset&&
    /^[a-f0-9]{64}$/.test(cp.anchorSha256??'')&&
    typeof cp.completeLastLine==='boolean';
}

// Reuse the same reverse scan for normal tails and appended evidence.
// One extra valid record establishes a real coverage limit without parsing
// every historical record.
function recentCapturedRecords(data,limit,discardLeading,incompleteTail) {
  const records=[];
  let end=incompleteTail?data.lastIndexOf('\n'):data.length;
  while(end>0 && records.length<limit) {
    const newline=data.lastIndexOf('\n',end-1);
    if(discardLeading && newline<0) break;
    const event=parseCapturedLine(data.slice(newline+1,end).replace(/\r$/,''));
    if(event) records.push(event);
    end=newline;
  }
  return records.reverse();
}

export async function readGameEvidence(directory,slot,maxRecords=EVIDENCE_LIMIT,{
  lstatFile=lstat,openFile=open,probeCheckpoint=null,probeOnly=false,
  evidenceCheckpoint=null,
}={}) {
  slotOf(slot);
  if(evidenceCheckpoint!==null&&(probeOnly||probeCheckpoint!==null)) {
    throw new TypeError('Choose one capture checkpoint mode');
  }
  const checkpoint=probeCheckpoint??evidenceCheckpoint;
  const prefix=evidenceCheckpoint!==null?'EVIDENCE':'PROBE';
  // The public read-only helper may be called outside the MCP allowlist.
  // A zero/negative/noninteger cap must not accidentally return the whole tail.
  if(!Number.isSafeInteger(maxRecords)||maxRecords<1) {
    throw new TypeError('Evidence record limit must be a positive integer');
  }
  const recordLimit=Math.min(maxRecords,EVIDENCE_LIMIT);
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
      // Normal analytics queries retain their original bounded tail.
      // Baseline preparation reads only the last 128-byte checkpoint anchor.
      // Probe or evidence-window reads reuse the appended bytes plus anchor.
      const size=Number(pinned.size);
      let bytes=Math.min(size,EVIDENCE_TAIL_BYTES);
      if(checkpoint!==null) {
        const cp=checkpoint;
        if(!validCaptureCursor(cp)) {
          return unstableEvidence(slot,prefix+'_CHECKPOINT_INVALID');
        }
        if(cp.byteOffset>size) {
          return unstableEvidence(slot,prefix+'_CAPTURE_TRUNCATED');
        }
        const appended=size-cp.byteOffset;
        if(appended+cp.anchorBytes>EVIDENCE_TAIL_BYTES) {
          return unstableEvidence(slot,prefix+'_CAPTURE_TAIL_GAP');
        }
        bytes=Math.min(size,Math.max(128,appended+cp.anchorBytes));
      } else if(probeOnly) {
        bytes=Math.min(size,128);
      }
      const start=size-bytes;
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
      // Checkpoint reads decode only appended bytes after continuity checks.
      const incompleteTrailingLineIgnored=
        buffer.length>0 && buffer[buffer.length-1]!==10;
      let records=probeOnly||checkpoint!==null?[]:
        recentCapturedRecords(buffer.toString('utf8'),recordLimit,start>0,
          incompleteTrailingLineIgnored);
      let windowCoverage=null;
      // Exact stable-file checkpoint. File timestamps cannot establish that
      // a Probe result belongs to the run prepared by this MCP process.
      const fileIdentity=createHash('sha256')
        .update(pinned.dev.toString()+':'+pinned.ino.toString())
        .digest('hex');
      const anchorBytes=Math.min(128,buffer.length);
      const captureCursor={
        kind:'STABLE_CAPTURE_CURSOR_V1',fileIdentity,
        byteOffset:Number(pinned.size),
        anchorBytes,
        anchorSha256:createHash('sha256')
          .update(buffer.subarray(buffer.length-anchorBytes)).digest('hex'),
        completeLastLine:buffer.length===0||buffer[buffer.length-1]===10,
      };
      let probeEvents=[];
      if(checkpoint!==null) {
        const cp=checkpoint;
        if(cp.fileIdentity!==fileIdentity) {
          return unstableEvidence(slot,prefix+'_CAPTURE_REPLACED');
        }
        if(cp.byteOffset>captureCursor.byteOffset) {
          return unstableEvidence(slot,prefix+'_CAPTURE_TRUNCATED');
        }
        if(cp.byteOffset-cp.anchorBytes<start) {
          return unstableEvidence(slot,prefix+'_CAPTURE_TAIL_GAP');
        }
        const from=cp.byteOffset-start;
        const anchor=buffer.subarray(from-cp.anchorBytes,from);
        if(createHash('sha256').update(anchor).digest('hex')!==cp.anchorSha256) {
          return unstableEvidence(slot,prefix+'_CAPTURE_PREFIX_CHANGED');
        }
        let added=buffer.subarray(from).toString('utf8');
        // A partial line present at preparation must never be completed
        // later and then mistaken for newly collected test evidence.
        if(!cp.completeLastLine) {
          const firstNewline=added.indexOf('\n');
          added=firstNewline<0?'':added.slice(firstNewline+1);
        }
        // Incomplete active writer tails are ignored until the next read.
        if(added && !added.endsWith('\n')) {
          added=added.slice(0,added.lastIndexOf('\n')+1);
        }
        if(evidenceCheckpoint!==null) {
          const scanned=recentCapturedRecords(added,recordLimit+1,false,false);
          const limited=scanned.length>recordLimit;
          records=limited?scanned.slice(-recordLimit):scanned;
          windowCoverage={recordLimitReached:limited,
            baselinePartialLineExcluded:!cp.completeLastLine};
        } else {
          probeEvents=added.split(/\r?\n/).map(parseProbeCapturedLine).filter(Boolean);
          if(probeEvents.length>200) {
            return unstableEvidence(slot,'PROBE_MARKER_VOLUME_EXCEEDED');
          }
        }
      }
      return {slot,status:'CAPTURE_FILE_PRESENT',
        events:records,
        captureCursor,probeEvents,
        attribution:'OPERATOR_NAMED_UNVERIFIED',
        truncated:windowCoverage!==null?windowCoverage.recordLimitReached:
          pinned.size>BigInt(EVIDENCE_TAIL_BYTES),
        ...(windowCoverage!==null?{windowCoverage}:{}),
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

