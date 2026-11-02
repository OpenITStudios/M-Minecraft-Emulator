// SOURCE-AUTHORED_ONLY. No tests/builds/gameplay executed on this revision.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {validateMapTestContract,planMapTests,compactMapTestPlan,
  readMapTestContractCached,mapContractSourceStatus} from './map-test-plan.mjs';
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
  assert.throws(()=>validateMapTestContract({...value,command:'/scriptevent'}));
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
    assert.equal(first.mapPlan.mapAcceptance,'NOT_TESTED');
    assert.equal(first.commandSentByLazyTester,false);
    assert.equal((await handler.callTool('probe_qa',{slot:2,scope:'full'})).reason,
      'QA_SCOPE_CHANGE_REQUIRES_RESTART');
    assert.equal((await handler.callTool('probe_qa',{slot:2})).status,'WAITING');
    await writeFile(file,JSON.stringify({...base(),revision:'spec-v2-changed'}));
    const blocked=await handler.callTool('probe_qa',{slot:2});
    assert.equal(blocked.reason,'MAP_CONTRACT_CHANGED_STOP');
    assert.equal(blocked.terminal,true);
    assert.equal((await handler.callTool('probe_qa',{slot:2})).reason,
      'MAP_CONTRACT_CHANGED_STOP');
    const restarted=await handler.callTool('probe_qa',{slot:2,restart:true,scope:'full'});
    assert.equal(restarted.mapPlan.revision,'spec-v2-changed');
  }finally{await rm(dir,{recursive:true,force:true});}
});
