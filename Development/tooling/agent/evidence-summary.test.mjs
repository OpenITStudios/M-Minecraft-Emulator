// Author only; execution requires the operator's later explicit permission.
import test from 'node:test';
import assert from 'node:assert/strict';
import {
  normalizeEvidenceSlots,summarizeEvidenceReports,summarizeGameEvidence,
} from './evidence-summary.mjs';

const event=(type,data,time='2026-10-30T12:00:00.000Z')=>({
  type, level:'log', observedAt:time, data,
});

test('slot selection refuses duplicates, missing slots, and out-of-range values',()=>{
  assert.deepEqual(normalizeEvidenceSlots([5,1,3]),[1,3,5]);
  for(const slots of [[],[1,1],[0],[1,6],'1',Array(6).fill(1)]) {
    assert.throws(()=>normalizeEvidenceSlots(slots));
  }
});

test('analytics export deduplicates stable record IDs across repeated pulls',()=>{
  const one={schema:1,type:'stage_attempt',record_id:'s1:player:stage:1',
    session_id:'s1',stage:'round1',end_reason:null,
    t:{tick:30},map:{name:'Probe'},data:{attempt:1}};
  const later={...one,end_reason:'fail',t:{tick:40}};
  const report={slot:2,status:'CAPTURE_FILE_PRESENT',events:[
    event('analytics',one),event('analytics',later,'2026-10-30T12:00:01.000Z')
  ]};
  const output=summarizeEvidenceReports([2],[report]);
  assert.equal(output.records,1);
  assert.equal(output.duplicatesSuppressed,1);
  assert.equal(output.timeline[0].endReason,'fail');
  assert.equal(output.timeline[0].severity,'ATTENTION');
  assert.equal(output.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
});

test('detects dropped analytics counters and final game results',()=>{
  const reports=[{slot:1,status:'CAPTURE_FILE_PRESENT',events:[
    event('analytics',{schema:1,type:'analytics_summary',session_id:'s1',
      dropped_counts:{item_use:2},failed_counts:{session:1}}),
    event('game_results',{schema:1,record_type:'gameplay_result',
      total_score:45,map:{name:'Test Map'},data:{canonical:{record_id:'r2'}}}),
  ]}];
  const data=summarizeEvidenceReports([1],reports);
  assert.equal(data.attentionCount,1);
  assert.equal(data.resultCount,1);
  assert.equal(data.timeline[1].score,45);
  assert.equal(data.worldIdentity,'NOT_VERIFIED');
});

test('missing capture never becomes an empty confirmed game session',async()=>{
  const data=await summarizeGameEvidence('C:\\agent-evidence',[1,3],
    async(_directory,slot)=>({slot,status:slot===1?'NO_CAPTURE_FILE':'NOT_CONFIGURED',events:[]}));
  assert.deepEqual(data.missingSlots,[1,3]);
  assert.equal(data.records,0);
  assert.equal(data.playerToProfileIdentity,'NOT_VERIFIED');
});

test('bad evidence file is isolated without preventing other profile reports',async()=>{
  const data=await summarizeGameEvidence('C:\\agent-evidence',[2,3],
    async(_directory,slot)=>{
      if(slot===2)throw new Error('private local data must not leak');
      return {slot,status:'CAPTURE_FILE_PRESENT',events:[
        event('analytics',{schema:1,type:'death',record_id:'d1',session_id:'s2'})
      ]};
    });
  assert.equal(data.sources[0].status,'UNAVAILABLE');
  assert.equal(data.sources[1].status,'CAPTURE_FILE_PRESENT');
  assert.equal(data.timeline[0].eventType,'death');
  assert.equal(JSON.stringify(data).includes('private local data'),false);
});

test('same record id from different gameplay sessions is not collapsed',()=>{
  const recordId='reused-record';
  const reports=[{slot:2,status:'CAPTURE_FILE_PRESENT',events:[
    event('analytics',{type:'stage_attempt',record_id:recordId,
      session_id:'session-A',stage:'round1'}),
    event('analytics',{type:'stage_attempt',record_id:recordId,
      session_id:'session-B',stage:'round2'},'2026-10-30T12:00:01.000Z'),
  ]}];
  const result=summarizeEvidenceReports([2],reports);
  assert.equal(result.records,2);
  assert.equal(result.duplicatesSuppressed,0);
  assert.deepEqual(result.timeline.map(e=>e.sessionId),
    ['session-A','session-B']);
  assert.equal(result.playerToProfileIdentity,'NOT_VERIFIED');
});

test('without verified session id repeated record ids remain separate',()=>{
  const raw={type:'stage_attempt',record_id:'unknown-session-record'};
  const result=summarizeEvidenceReports([2],[{
    slot:2,status:'CAPTURE_FILE_PRESENT',events:[
      event('analytics',raw),
      event('analytics',{...raw,stage:'later'},'2026-10-30T12:00:01.000Z'),
    ],
  }]);
  assert.equal(result.records,2);
  assert.equal(result.duplicatesSuppressed,0);
  assert.equal(result.timeline[0].sessionId,null);
});
