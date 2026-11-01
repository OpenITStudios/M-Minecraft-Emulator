// Authored source fixtures only. Do not execute until separately approved.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,readFile,writeFile,readdir,rm,symlink} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {
  validateDiagnosticsRoot,analyzeDiagnosticJsonl,importDiagnostics,
  diagnosticsOverview,listDiagnosticIssues,getDiagnosticIssue,
  queryDiagnosticEvidence,
} from './diagnostics-local.mjs';

const fixture=(run='td-test')=>[
  {schema:1,run,seq:1,tick:100,kind:'state_change',arena:'arena-02',
    check:'wave.phase',expected:'WAITING',actual:'RUNNING'},
  {schema:1,run,seq:2,tick:101,kind:'assertion',arena:'arena-02',
    check:'flag.player_damage_blocked',expected:350,actual:330},
  {schema:1,run,seq:3,tick:102,kind:'assertion',arena:'arena-02',
    check:'flag.player_damage_blocked',expected:350,actual:320},
  {schema:1,run,seq:4,tick:103,kind:'assertion',arena:'arena-03',
    check:'flag.player_damage_blocked',expected:350,actual:350},
  {schema:1,run,seq:6,tick:104,kind:'script_error',arena:'arena-02',
    code:'SCRIPT_EXCEPTION'},
].map(x=>JSON.stringify(x)).join('\n')+'\n';

test('pure analysis groups issue candidates but never claims accepted gameplay',()=>{
  const report=analyzeDiagnosticJsonl(fixture(),'td-test');
  assert.equal(report.totals.events,5);
  assert.equal(report.totals.mismatchObservations,2);
  assert.equal(report.totals.matchedObservations,1);
  assert.equal(report.totals.scriptErrors,1);
  assert.equal(report.totals.telemetryGaps,1);
  assert.equal(report.issueCount,3);
  assert.equal(report.issues[0].occurrences,2);
  assert.equal(report.issues[0].arena,'arena-02');
  assert.equal(report.verdict,'CANDIDATES_ONLY_NO_GAMEPLAY_ACCEPTANCE');
  assert.equal(JSON.stringify(report).includes('PASS'),false);
});

test('untrusted or partial telemetry cannot be promoted to valid evidence',()=>{
  const run='td-test',base={schema:1,run,seq:1,tick:10,
    kind:'assertion',check:'shop.purchase',expected:30,actual:50};
  const bad=[
    {...base,account:'user@example.com'},
    {...base,expected:{secret:'value'}},
    {...base,actual:'private account'},
    {...base,seq:0},
    {...base,run:'wrong'},
    {...base,kind:'unknown'},
  ];
  for(const event of bad) {
    assert.throws(()=>analyzeDiagnosticJsonl(JSON.stringify(event)+'\n',run));
  }
  assert.throws(()=>analyzeDiagnosticJsonl(JSON.stringify(base),run));
  assert.throws(()=>analyzeDiagnosticJsonl(
    [base,{...base,seq:1}].map(JSON.stringify).join('\n')+'\n',run));
  assert.throws(()=>analyzeDiagnosticJsonl(
    [base,{...base,seq:2,tick:9}].map(JSON.stringify).join('\n')+'\n',run));
  assert.throws(()=>validateDiagnosticsRoot('/tmp/unrelated'));
  assert.throws(()=>analyzeDiagnosticJsonl('raw log without a schema\n',run));
});

test('explicit local store retains full capture, MCP-style queries reveal only bounded evidence',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-local-diag-'));
  const data=path.join(root,'Data');
  await mkdir(data);
  const store=path.join(data,'Diagnostics');
  const source=path.join(root,'source.jsonl');
  await writeFile(source,fixture());
  try{
    assert.equal((await diagnosticsOverview(undefined)).status,'NOT_CONFIGURED');
    assert.equal((await diagnosticsOverview(store)).status,'NO_LOCAL_DIAGNOSTICS');
    const imported=await importDiagnostics({store,source,run:'td-test'});
    assert.equal(imported.status,'LOCAL_ARCHIVED_UNVERIFIED');
    assert.equal(imported.rawDataSharedWithAI,false);
    const retained=await readFile(path.join(store,'runs','td-test','events.jsonl'),'utf8');
    assert.equal(retained,fixture());
    const overview=await diagnosticsOverview(store,'td-test');
    assert.equal(overview.issueCount,3);
    assert.equal(overview.gameplay,'NOT_VERIFIED');
    const runs=await diagnosticsOverview(store);
    assert.deepEqual(runs.runs,['td-test']);
    const first=await listDiagnosticIssues(store,'td-test',{limit:1});
    assert.equal(first.issues.length,1);
    assert.equal(typeof first.nextCursor,'number');
    const second=await listDiagnosticIssues(store,'td-test',
      {cursor:first.nextCursor,limit:2});
    assert.equal(second.issues.length,2);
    const id=first.issues[0].id;
    const detail=await getDiagnosticIssue(store,'td-test',id);
    assert.equal(detail.issue.occurrences,2);
    assert.equal(detail.issue.confidence,'UNVERIFIED_IMPORT_CANDIDATE');
    const evidence=await queryDiagnosticEvidence(store,'td-test',id,{limit:1});
    assert.equal(evidence.total,2);
    assert.equal(evidence.records.length,1);
    assert.equal(evidence.records[0].expected,350);
    assert.equal(typeof evidence.nextCursor,'number');
    assert.equal(JSON.stringify(evidence).includes('source.jsonl'),false);
    assert.equal(JSON.stringify(evidence).includes('user@example.com'),false);
    const more=await queryDiagnosticEvidence(store,'td-test',id,
      {cursor:evidence.nextCursor,limit:2});
    assert.equal(more.records.length,1);
    assert.equal(more.nextCursor,null);
    await assert.rejects(()=>importDiagnostics({store,source,run:'td-test'}));
    assert.deepEqual(await readdir(path.join(store,'runs')),['td-test']);
    assert.equal(await readFile(path.join(store,'runs','td-test','events.jsonl'),'utf8'),
      fixture());
  } finally {await rm(root,{recursive:true,force:true});}
});

test('invalid import never creates a local store or touches source evidence',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-local-diag-invalid-'));
  const data=path.join(root,'Data');await mkdir(data);
  const source=path.join(root,'bad.jsonl');
  await writeFile(source,'{not valid JSON}\n');
  const store=path.join(data,'Diagnostics');
  try{
    await assert.rejects(()=>importDiagnostics({store,source,run:'test-run'}));
    assert.equal((await diagnosticsOverview(store)).status,'NO_LOCAL_DIAGNOSTICS');
    assert.equal(await readFile(source,'utf8'),'{not valid JSON}\n');
  }finally{await rm(root,{recursive:true,force:true});}
});

test('store refuses symlinked Data parent, without following outside paths',async(t)=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-local-diag-link-'));
  const actual=path.join(root,'real');await mkdir(actual);
  const linked=path.join(root,'Data');
  try{
    try{await symlink(actual,linked,'dir');}
    catch{t.skip('Symlinks unavailable in this environment');return;}
    const source=path.join(root,'input.jsonl');
    await writeFile(source,fixture());
    await assert.rejects(()=>importDiagnostics({
      store:path.join(linked,'Diagnostics'),source,run:'td-test',
    }));
  }finally{await rm(root,{recursive:true,force:true});}
});

test('query caps and invalid identifiers refuse broad/unsafe extraction',async()=>{
  await assert.rejects(()=>listDiagnosticIssues('/tmp/Data/Diagnostics','bad/run'));
  await assert.rejects(()=>queryDiagnosticEvidence('/tmp/Data/Diagnostics',
    'valid-run','../hidden',{limit:999}));
  await assert.rejects(()=>listDiagnosticIssues('/tmp/Data/Diagnostics',
    'valid-run',{limit:1000}));
});
