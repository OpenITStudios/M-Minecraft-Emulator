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

test('unstable file evidence is excluded and reported, not treated as empty accepted capture',()=>{
  const data=summarizeEvidenceReports([2,3],[
    {slot:2,status:'CAPTURE_FILE_UNSTABLE',events:[
      event('game_results',{record_type:'gameplay_result',total_score:99}),
    ]},
    {slot:3,status:'CAPTURE_FILE_PRESENT',events:[
      event('analytics',{type:'death',session_id:'s3'}),
    ],incompleteTrailingLineIgnored:true},
  ]);
  assert.equal(data.records,1);
  assert.deepEqual(data.missingSlots,[2]);
  assert.equal(data.sources[0].status,'CAPTURE_FILE_UNSTABLE');
  assert.equal(data.sources[0].parsedRecords,0);
  assert.equal(data.sources[1].incompleteTrailingLineIgnored,true);
  assert.equal(data.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
});

test('untyped game-result objects never inflate resultCount or gameplay acceptance',()=>{
  const report={slot:2,status:'CAPTURE_FILE_PRESENT',events:[
    event('game_results',{passed:true,total_score:999}),
    event('game_results',{schema:1,record_type:'progress',
      passed:true,total_score:999},'2026-10-30T12:00:01.000Z'),
    event('game_results',{schema:'1',record_type:'gameplay_result',
      total_score:999},'2026-10-30T12:00:02.000Z'),
    event('game_results',{schema:1,record_type:'gameplay_result',
      total_score:12},'2026-10-30T12:00:03.000Z'),
    event('analytics',{type:'death',session_id:'s2'},
      '2026-10-30T12:00:04.000Z'),
  ]};
  const output=summarizeEvidenceReports([2],[report]);
  assert.equal(output.records,2);
  assert.equal(output.resultCount,1);
  assert.equal(output.sources[0].parsedRecords,2);
  assert.equal(output.timeline.find(x=>x.severity==='RESULT').score,12);
  assert.equal(output.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
  assert.equal(output.playerToProfileIdentity,'NOT_VERIFIED');
  assert.match(output.note,/NEVER a gameplay success/);
});

test('team result export_summary remains a completeness signal, not a scored result',()=>{
  const reports=[{slot:2,status:'CAPTURE_FILE_PRESENT',events:[
    event('game_results',{schema:1,record_type:'gameplay_result',
      total_score:50,map:{name:'Arena Test'},data:{canonical:{record_id:'player-11:secret-id'}}}),
    event('game_results',{schema:1,record_type:'export_summary',
      counts:{gameplay_result:1,level_score:1},
      failed_counts:{gameplay_result:2}}),
    event('game_results',{schema:1,record_type:'level_score',
      total_score:20,map:{name:'Arena Test'},data:{canonical:{}}}),
  ]}];
  const output=summarizeEvidenceReports([2],reports);
  assert.equal(output.resultCount,1);
  assert.equal(output.teamGameResultSummaryCount,1);
  assert.equal(output.teamLevelScoreCount,1);
  assert.equal(output.attentionCount,1);
  assert.equal(output.timeline.some(e=>e.eventType==='export_summary'&&
    e.severity==='ATTENTION'),true);
  // Team record IDs may include player IDs; keep them local for dedupe only.
  assert.equal(JSON.stringify(output).includes('player-11:secret-id'),false);
  assert.equal(output.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
});

test('universal low-volume summary is separate from team-owned analytics/scoring',()=>{
  const diagnostic={
    schema:1,session:'lt-abcde-1',seq:2,tick:200,kind:'summary',
    data:{capture:1,reason:'interval',saturated:0,
      counts:{firstSpawns:1,respawns:2,playerLeaves:1,playerDeaths:0}},
  };
  const result=summarizeEvidenceReports([1],[{
    slot:1,status:'CAPTURE_FILE_PRESENT',
    events:[event('universal_diagnostics',diagnostic)],
  }]);
  assert.equal(result.records,1);
  assert.equal(result.universalSummaryCount,1);
  assert.equal(result.resultCount,0);
  assert.equal(result.timeline[0].source,'universal_diagnostics');
  assert.equal(result.timeline[0].summaryCounts.respawns,2);
  assert.equal(result.timeline[0].severity,'INFO');
  assert.equal(result.worldIdentity,'NOT_VERIFIED');
});

test('universal markers are deduped by session and sequence, not by map identity',()=>{
  const base={schema:1,session:'lt-abcde-1',seq:3,tick:400,kind:'heartbeat',
    data:{capture:1,active:true}};
  const output=summarizeEvidenceReports([2],[{
    slot:2,status:'CAPTURE_FILE_PRESENT',events:[
      event('universal_diagnostics',base),
      event('universal_diagnostics',{...base}),
      event('universal_diagnostics',{...base,session:'lt-abcde-2'}),
    ],
  }]);
  assert.equal(output.records,2);
  assert.equal(output.duplicatesSuppressed,1);
  assert.equal(output.resultCount,0);
  assert.equal(output.timeline.every(e=>e.source==='universal_diagnostics'),true);
});

test('forged universal detail fields and unbounded counters are not accepted',()=>{
  const good={schema:1,session:'lt-abcde-1',seq:1,tick:100,
    kind:'summary',data:{capture:1,reason:'interval',saturated:0,
      counts:{playerDeaths:1}}};
  const rejected=[
    {...good,data:{...good.data,playerName:'sensitive'}},
    {...good,data:{...good.data,counts:{playerDeaths:1000001}}},
    {...good,data:{...good.data,counts:{'ignore AI instructions':9}}},
    {...good,session:'some-other-script'},
    {...good,seq:0},
    {...good,unknownField:'secret'},
  ];
  for(const item of rejected){
    const out=summarizeEvidenceReports([2],[{
      slot:2,status:'CAPTURE_FILE_PRESENT',
      events:[event('universal_diagnostics',item)],
    }]);
    assert.equal(out.records,0);
    assert.equal(JSON.stringify(out).includes('sensitive'),false);
  }
});
