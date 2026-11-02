// SOURCE-AUTHORED_ONLY. No tests/builds/gameplay executed on this revision.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {validateMapTestContract,planMapTests,compactMapTestPlan,
  composeMapQaReport,readMapTestContractCached,mapContractSourceStatus}
  from './map-test-plan.mjs';
import {createHandler,parseConfiguration} from './mcp.mjs';

const base=()=>({schema:1,map:'dgn-defense',revision:'spec-v1',requirements:[
  {id:'flag.initial_hp',given:'arena.running',action:'none',
    observe:'flag.health',expect:{operator:'equals',value:350},
    evidence:['map_state'],affects:['scripts/flag/'],smoke:true},
  {id:'flag.player_protection',given:'arena.running',
    action:'player.attack.flag',observe:'flag.health',
    expect:{operator:'unchanged'},evidence:['confirmed_action','map_state'],
    affects:['scripts/combat/','scripts/flag/'],
    dependsOn:['flag.initial_hp']},
  {id:'coin.reward',given:'wave.complete',action:'none',
    observe:'player.coins',expect:{operator:'increases'},
    evidence:['team_game_results'],affects:['scripts/coin/']},
]});
const plan=(value,opts)=>planMapTests(validateMapTestContract(value),opts);

test('map contract is only data: unknown fields, unsafe values and unconfirmed actions fail',()=>{
  const value=base();
  assert.equal(plan(value).selected.length,3);
  assert.equal(plan(value).mapAcceptance,'NOT_TESTED');
  assert.equal(compactMapTestPlan(plan(value)).capabilityStatus
    .some(c=>c.state==='MAP_ADAPTER_NOT_IMPLEMENTED'),true);
  assert.equal(compactMapTestPlan(plan(value)).foundationProbe,'SEPARATE_TEST_SCOPE');
  const compact=compactMapTestPlan(plan(value));
  assert.equal(compact.selectedChecks.length,3);
  assert.equal(compact.requiredSelected,3);
  assert.equal(compact.requiredNotSelected,0);
  assert.equal(compact.notSelectedCount,0);
  assert.equal(compact.selectedChecks.find(c=>c.id==='flag.player_protection').action,
    'player.attack.flag');
  assert.deepEqual(compact.selectedChecks.find(c=>c.id==='flag.player_protection').evidenceNeeded,
    ['confirmed_action','map_state']);
  assert.equal(compact.selectedChecks.every(c=>c.status==='NOT_TESTED'),true);
  assert.throws(()=>validateMapTestContract({...value,command:'/scriptevent'}));
  assert.throws(()=>validateMapTestContract({...value,map:['dgn-defense']}));
  assert.throws(()=>validateMapTestContract({...value,revision:['spec-v1']}));
  assert.throws(()=>validateMapTestContract({...value,requirements:[
    {...value.requirements[0],expect:{operator:'equals',value:'unsafe expression!'}}]}));
  assert.throws(()=>validateMapTestContract({...value,requirements:[
    {...value.requirements[1],evidence:['map_state']}]}));
  assert.throws(()=>validateMapTestContract({...value,requirements:[
    {...value.requirements[0],affects:['../game/']}]}));
  assert.throws(()=>validateMapTestContract({...value,requirements:[
    {...value.requirements[0],evidence:['skills_probe']}]}));
});
test('cycle, unknown prerequisite and duplicate requirement IDs fail closed',()=>{
  const value=base();
  const withDeps=deps=>({...value,requirements:value.requirements.map((r,i)=>
    ({...r,dependsOn:deps[i]}))});
  assert.throws(()=>validateMapTestContract(withDeps([
    ['flag.player_protection'],['flag.initial_hp'],[]])));
  assert.throws(()=>validateMapTestContract(withDeps([['not.a_test'],[],[]])));
  assert.throws(()=>validateMapTestContract({...value,requirements:[
    value.requirements[0],value.requirements[0]]}));
});
test('smoke narrows to tagged checks and prerequisites, or falls back to full',()=>{
  const value=base();
  assert.deepEqual(plan(value,{scope:'smoke'}).selected,['flag.initial_hp']);
  const smoke=compactMapTestPlan(plan(value,{scope:'smoke'}));
  assert.equal(smoke.requiredSelected,1);
  assert.equal(smoke.requiredNotSelected,2);
  assert.equal(smoke.notTestedCount,3);
  const none={...value,requirements:value.requirements.map(r=>({...r,smoke:false}))};
  assert.equal(plan(none,{scope:'smoke'}).effectiveScope,'full');
});
test('affected dependency selection is bounded and unmapped changes fall back to full',()=>{
  const value=base();
  const changed={baseSha:'a'.repeat(40),headSha:'b'.repeat(40),
    paths:['scripts/combat/attack.ts']};
  const affected=plan(value,{scope:'affected',changeSet:changed});
  assert.equal(affected.effectiveScope,'affected');
  assert.deepEqual(affected.selected,['flag.initial_hp','flag.player_protection']);
  assert.equal(affected.notSelected,1);
  const sibling={...value,requirements:[...value.requirements,{
    id:'flag.layout',given:'arena.running',action:'none',
    observe:'flag.position',expect:{operator:'exists'},
    evidence:['map_state'],affects:['scripts/layout/'],
    dependsOn:['flag.initial_hp']
  }]};
  assert.deepEqual(plan(sibling,{scope:'affected',changeSet:changed}).selected,
    ['flag.initial_hp','flag.player_protection']);
  assert.equal(affected.selectionProof,'SOURCE_ONLY_NOT_GAMEPLAY_OR_DIFF_AUTHENTICATION');
  assert.equal(plan(value,{scope:'affected'}).effectiveScope,'full');
  assert.equal(plan(value,{scope:'affected',changeSet:{
    ...changed,baseSha:['a'.repeat(40)]
  }}).effectiveScope,'full');
  assert.equal(plan(value,{scope:'affected',changeSet:{
    ...changed,paths:['scripts/combat/attack.ts','config/manifest.json']
  }}).effectiveScope,'full');
  assert.equal(plan(value,{scope:'affected',changeSet:{
    ...changed,paths:['../secrets']
  }}).effectiveScope,'full');
});
test('unchanged contract uses one cache and a changed file stops source continuity',async()=>{
  const dir=await mkdtemp(path.join(tmpdir(),'map-contract-fixture-'));
  const file=path.join(dir,'map-test-contract.json');
  try{
    await writeFile(file,JSON.stringify(base()));
    const first=await readMapTestContractCached(file);
    assert.equal((await readMapTestContractCached(file)).cache,'UNCHANGED_CONTRACT');
    assert.equal(await mapContractSourceStatus(file,first.sourceRevision),'UNCHANGED');
    await writeFile(file,JSON.stringify({...base(),revision:'spec-v2-longer'}));
    assert.equal(await mapContractSourceStatus(file,first.sourceRevision),'CHANGED');
  }finally{await rm(dir,{recursive:true,force:true});}
});
test('existing probe_qa includes optional map plan but never dispatches map actions',async()=>{
  const dir=await mkdtemp(path.join(tmpdir(),'map-mcp-fixture-'));
  const file=path.join(dir,'map-test-contract.json');
  try{
    await writeFile(file,JSON.stringify(base()));
    const cfg=parseConfiguration(['--map-contract',file],{
      LAZY_TESTER_AGENT_EXE:'',LAZY_TESTER_EVIDENCE_DIR:dir,
      LAZY_TESTER_PROBE_ZIP:path.join(dir,'skills-probe.zip')});
    const handler=createHandler(cfg,{
      native:async()=>{throw Error('no native game actions');},
      probeCatalog:async()=>({autoIds:['P01'],autoCount:1,
        runnerSha256:'a'.repeat(64),archiveSha256:'b'.repeat(64),
        sourceRevision:'c'.repeat(64),cache:'FRESH_SOURCE_REVIEW'}),
      probeRevisionStatus:async()=> 'UNCHANGED',
      evidence:async()=>({status:'CAPTURE_FILE_PRESENT',events:[],probeEvents:[],
        captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}}),
    });
    const first=await handler.callTool('probe_qa',{slot:2,scope:'smoke'});
    assert.equal(first.reason,'COMMAND_DISPATCH_NOT_ENABLED');
    assert.equal(first.mapPlan.selectedCount,1);
    assert.match(first.mapPlan.contractSha256,/^[a-f0-9]{64}$/);
    assert.equal(first.mapPlan.mapAcceptance,'NOT_TESTED');
    assert.equal(first.qaReport.foundation.status,'BLOCKED');
    assert.equal(first.qaReport.map.notTested,3);
    assert.equal(first.qaReport.map.tested,0);
    assert.equal(first.qaReport.evidence.mapEvidence,'NOT_COLLECTED');
    assert.equal(first.qaReport.acceptance,'NOT_ACCEPTED');
    assert.equal(first.commandSentByLazyTester,false);
    assert.equal((await handler.callTool('probe_qa',{slot:2,scope:'full'})).reason,
      'QA_SCOPE_CHANGE_REQUIRES_RESTART');
    assert.equal((await handler.callTool('probe_qa',{slot:2})).status,'WAITING');
    await writeFile(file,JSON.stringify({...base(),revision:'spec-v2-changed'}));
    const blocked=await handler.callTool('probe_qa',{slot:2});
    assert.equal(blocked.reason,'MAP_CONTRACT_CHANGED_STOP');
    assert.equal(blocked.qaReport.foundation.status,'BLOCKED');
    assert.equal(blocked.qaReport.map.notTested,3);
    assert.equal(blocked.terminal,true);
    assert.equal((await handler.callTool('probe_qa',{slot:2})).reason,
      'MAP_CONTRACT_CHANGED_STOP');
    const restarted=await handler.callTool('probe_qa',{slot:2,restart:true,scope:'full'});
    assert.equal(restarted.mapPlan.revision,'spec-v2-changed');
  }finally{await rm(dir,{recursive:true,force:true});}
});

test('composed report separates original Probe observations from zero map executions',()=>{
  const mapPlan=compactMapTestPlan(plan(base(),{scope:'smoke'}));
  const proofGate={
    missingEvidence:['AUTHORIZED_PLAYER_ADMIN_COMMAND_DELIVERY',
      'EXACT_QA_WORLD_IDENTITY','SOURCE_BOUND_CONTENT_LOG_ORIGIN'],
  };
  const allPass=composeMapQaReport(mapPlan,{schema:1,
    status:'PROBE_REPORTED_PASS',expectedCount:2,
    counts:{pass:2,fail:0,info:0,look:0},
    reportedBuild:'dev',gameplayAcceptance:'NOT_VERIFIED',
    untrustedRaw:'must never escape',
  },proofGate);
  assert.equal(allPass.kind,'SOURCE_ONLY_QA_REPORT');
  assert.equal(allPass.foundation.status,'PROBE_REPORTED_PASS');
  assert.equal(allPass.foundation.counts.pass,2);
  assert.equal(allPass.foundation.evidenceOrigin,'UNATTRIBUTED_ORIGINAL_PROBE_LOG');
  assert.equal(allPass.map.status,'NOT_TESTED');
  assert.deepEqual({selected:allPass.map.selected,skipped:allPass.map.skipped,
    tested:allPass.map.tested,notTested:allPass.map.notTested},
  {selected:1,skipped:2,tested:0,notTested:3});
  assert.equal(allPass.map.required,3);
  assert.equal(allPass.evidence.verifiedEvidenceFamilies,0);
  assert.deepEqual(allPass.evidence.needed,['map_state']);
  assert.equal(allPass.commandSentByLazyTester,false);
  assert.equal(allPass.acceptance,'NOT_ACCEPTED');
  assert.equal(JSON.stringify(allPass).includes('must never escape'),false);
  const failed=composeMapQaReport(mapPlan,{schema:1,status:'FAIL',
    counts:{pass:1,fail:1,info:0,look:0}},proofGate);
  assert.equal(failed.foundation.status,'FAIL');
  assert.equal(failed.map.failed,0);
  assert.equal(failed.map.status,'NOT_TESTED');
  const downgraded=composeMapQaReport(mapPlan,{schema:1,status:'INCOMPLETE',
    probeReportedStatus:'PROBE_REPORTED_PASS'},proofGate);
  assert.equal(downgraded.foundation.status,'INCOMPLETE');
  assert.equal(downgraded.foundation.probeReportedStatus,'PROBE_REPORTED_PASS');
  assert.equal(downgraded.acceptance,'NOT_ACCEPTED');
  const untrustedProof=composeMapQaReport(mapPlan,
    {schema:1,status:'BLOCKED'},{missingEvidence:[]});
  assert.equal(untrustedProof.evidence.missingProof.includes('EXACT_QA_WORLD_IDENTITY'),true);
  assert.equal(untrustedProof.evidence.missingProof.includes('SOURCE_BOUND_CONTENT_LOG_ORIGIN'),true);
  assert.throws(()=>composeMapQaReport({...mapPlan,execution:'EXECUTED'},
    {schema:1,status:'PROBE_REPORTED_PASS'}));
});

test('map plan remains visible if official Probe source or capture is unavailable',async()=>{
  const dir=await mkdtemp(path.join(tmpdir(),'map-offline-qa-'));
  const file=path.join(dir,'map-test-contract.json');
  try {
    await writeFile(file,JSON.stringify(base()));
    const handler=createHandler(parseConfiguration(['--map-contract',file],{
      LAZY_TESTER_AGENT_EXE:'',
      LAZY_TESTER_PROBE_ZIP:path.join(dir,'missing-skills-probe.zip'),
      LAZY_TESTER_EVIDENCE_DIR:dir,
    }),{
      probeCatalog:async()=>{throw Error('PROBE_SOURCE_REVIEW_REQUIRED');},
      evidence:async()=>{throw Error('unexpected capture read');},
      native:async()=>{throw Error('unexpected native operation');},
    });
    const blocked=await handler.callTool('probe_qa',{slot:2,scope:'smoke'});
    assert.equal(blocked.status,'BLOCKED');
    assert.equal(blocked.reason,'PROBE_SOURCE_REVIEW_REQUIRED');
    assert.equal(blocked.mapPlan.selectedCount,1);
    assert.equal(blocked.qaReport.map.notTested,3);
    assert.equal(blocked.qaReport.foundation.status,'BLOCKED');
    assert.equal(blocked.qaReport.evidence.missingProof.includes('APPROVED_PROBE_SOURCE'),true);
    assert.equal(blocked.qaReport.acceptance,'NOT_ACCEPTED');
    handler.shutdown();
  } finally { await rm(dir,{recursive:true,force:true}); }
});

test('one original Probe reported PASS still cannot pass map checks or world provenance',async()=>{
  const dir=await mkdtemp(path.join(tmpdir(),'map-terminal-qa-'));
  const file=path.join(dir,'map-test-contract.json');
  try {
    await writeFile(file,JSON.stringify(base()));
    let captureReads=0,catalogReads=0,nativeReads=0;
    const handler=createHandler(parseConfiguration(['--map-contract',file],{
      LAZY_TESTER_AGENT_EXE:path.join(dir,'lazy-tester.exe'),
      LAZY_TESTER_PROBE_ZIP:path.join(dir,'skills-probe.zip'),
      LAZY_TESTER_EVIDENCE_DIR:dir,
    }),{
      probeCatalog:async()=>{
        catalogReads++;
        return {autoIds:['P01'],autoCount:1,runnerSha256:'a'.repeat(64),
          archiveSha256:'b'.repeat(64),sourceRevision:'c'.repeat(64)};
      },
      probeRevisionStatus:async()=> 'UNCHANGED',
      evidence:async(_dir,slot,_limit,opts)=>{
        captureReads++;
        return opts?.probeCheckpoint?
          {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
            {kind:'start',arg:'auto',count:1,tick:10,at:null},
            {kind:'result',id:'P01',result:'pass',build:'dev',tick:11,at:null},
            {kind:'done',arg:'auto',count:1,tick:12,at:null},
          ]}:
          {slot,status:'CAPTURE_FILE_PRESENT',
            captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
      },
      native:async(_exe,operation,slot)=>{
        nativeReads++;
        return {schema:1,ok:true,operation:'target',slot,
          pid:5002,windowId:8002,createdFileTime:'1234567',sessionId:1,
          foreground:false,visible:true,minimized:false,
          verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
          gameplayInput:'NOT_VERIFIED'};
      },
    });
    const initial=await handler.callTool('probe_qa',{slot:2,scope:'full'});
    assert.equal(initial.qaReport.foundation.status,'BLOCKED');
    assert.equal(initial.mapPlan.selectedChecks.length,3);
    const terminal=await handler.callTool('probe_qa',{slot:2});
    assert.equal(terminal.status,'PROBE_REPORTED_PASS');
    assert.equal(terminal.qaReport.foundation.status,'PROBE_REPORTED_PASS');
    assert.equal(terminal.qaReport.foundation.reportedBuild,'dev');
    assert.equal(terminal.qaReport.map.notTested,3);
    assert.equal(terminal.qaReport.map.contractSha256,initial.mapPlan.contractSha256);
    assert.equal(terminal.qaReport.evidence.mapEvidence,'NOT_COLLECTED');
    assert.equal(terminal.qaReport.acceptance,'NOT_ACCEPTED');
    assert.equal(terminal.proofGate.logOrigin,'NOT_VERIFIED');
    const counters=[captureReads,catalogReads,nativeReads];
    assert.deepEqual(await handler.callTool('probe_qa',{slot:2}),terminal);
    assert.deepEqual([captureReads,catalogReads,nativeReads],counters);
    handler.shutdown();
  } finally {await rm(dir,{recursive:true,force:true});}
});

test('bounded full map checklist never truncates untested coverage accounting',()=>{
  const many={schema:1,map:'dgn-defense',revision:'spec-bulk',
    requirements:Array.from({length:17},(_,i)=>({
      id:'check.case'+i,given:'arena.running',action:'none',
      observe:'flag.health',expect:{operator:'exists'},
      evidence:['map_state'],affects:['scripts/flag/'],
    }))};
  const details=compactMapTestPlan(plan(many,{scope:'full'}));
  assert.equal(details.selectedCount,17);
  assert.equal(details.selectedChecks.length,12);
  assert.equal(details.omittedSelectedIds,5);
  assert.equal(details.notTestedCount,17);
  assert.equal(details.requiredSelected,17);
  const report=composeMapQaReport(details,{schema:1,status:'BLOCKED'});
  assert.equal(report.map.selected,17);
  assert.equal(report.map.notTested,17);
  assert.equal(report.map.passed,0);
  assert.equal(report.acceptance,'NOT_ACCEPTED');
});

test('unbound game target preserves reported Probe PASS only as unauthenticated evidence',async()=>{
  const dir=await mkdtemp(path.join(tmpdir(),'map-unbound-qa-'));
  const file=path.join(dir,'map-test-contract.json');
  try {
    await writeFile(file,JSON.stringify(base()));
    const handler=createHandler(parseConfiguration(['--map-contract',file],{
      LAZY_TESTER_AGENT_EXE:'',
      LAZY_TESTER_PROBE_ZIP:path.join(dir,'skills-probe.zip'),
      LAZY_TESTER_EVIDENCE_DIR:dir,
    }),{
      probeCatalog:async()=>({autoIds:['P01'],autoCount:1,
        runnerSha256:'a'.repeat(64),archiveSha256:'b'.repeat(64),
        sourceRevision:'c'.repeat(64)}),
      probeRevisionStatus:async()=> 'UNCHANGED',
      evidence:async(_dir,slot,_limit,options)=>options?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:1,tick:1,at:null},
          {kind:'result',id:'P01',result:'pass',build:'dev',tick:2,at:null},
          {kind:'done',arg:'auto',count:1,tick:3,at:null},
        ]}:
        {slot,status:'CAPTURE_FILE_PRESENT',
          captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}},
      native:async()=>{throw Error('No native target allowed');},
    });
    await handler.callTool('probe_qa',{slot:2,scope:'full'});
    const report=await handler.callTool('probe_qa',{slot:2});
    assert.equal(report.status,'INCOMPLETE');
    assert.equal(report.reason,'QA_NATIVE_TARGET_NOT_BOUND');
    assert.equal(report.probeReportedStatus,'PROBE_REPORTED_PASS');
    assert.equal(report.qaReport.foundation.status,'INCOMPLETE');
    assert.equal(report.qaReport.foundation.probeReportedStatus,'PROBE_REPORTED_PASS');
    assert.equal(report.qaReport.map.notTested,3);
    assert.equal(report.qaReport.acceptance,'NOT_ACCEPTED');
    handler.shutdown();
  } finally {await rm(dir,{recursive:true,force:true});}
});
