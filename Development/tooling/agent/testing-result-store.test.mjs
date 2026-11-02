// Authored only, NOT_EXECUTED. No Minecraft, Windows or CI.
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,readFile,writeFile,readdir,rm,symlink} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {validateTestingArchiveRoot,archiveTestingResult,readTestingArchive,
  archivedTestingSlotReference} from './testing-result-store.mjs';

const id='a'.repeat(32),digest='c'.repeat(64);
const original=(status='PROBE_REPORTED_PASS')=>({
  schema:1,slot:2,terminal:true,status,
  gameplayAcceptance:'NOT_VERIFIED',
  ...(status==='PROBE_REPORTED_PASS'?{
    expectedCount:2,counts:{pass:2,fail:0,info:0,look:0},
  }:{}),
  proofGate:{logOrigin:'NOT_VERIFIED'},
});

test('explicit private Testing root only; source archive never includes raw logs',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-testing-store-'));
  const data=path.join(root,'Data');await mkdir(data);
  const store=path.join(data,'Testing');
  try {
    assert.equal(validateTestingArchiveRoot(store),store);
    assert.throws(()=>validateTestingArchiveRoot(path.join(root,'Other')));
    const raw={...original(),untrustedPrivatePath:'C:\\Users\\secret',
      capturedLog:'[GAME_RESULTS] private data'};
    const first=await archiveTestingResult({
      store,runId:id,slot:2,report:raw,reportDigest:digest});
    assert.equal(first.status,'ARCHIVED_SOURCE_ONLY');
    assert.equal(first.reportedOutcome,'PROBE_REPORTED_PASS');
    assert.deepEqual(first.reportedCounts,
      {expected:2,pass:2,fail:0,info:0,look:0});
    assert.equal(first.gameplayAcceptance,'NOT_VERIFIED');
    const text=await readFile(path.join(store,'runs',id,'report.json'),'utf8');
    assert.equal(text.includes('Users'),false);
    assert.equal(text.includes('capturedLog'),false);
    assert.equal(text.includes('GAME_RESULTS'),false);
    const second=await archiveTestingResult({
      store,runId:id,slot:2,report:raw,reportDigest:digest});
    assert.deepEqual(second,first,'Same run/digest never makes a second result');
    assert.deepEqual(await readdir(path.join(store,'runs')),[id]);
    const newReader=await readTestingArchive({store,runId:id});
    assert.deepEqual(newReader,first,'A new MCP process can recover a SOURCE-ONLY summary');
    await assert.rejects(()=>archiveTestingResult({
      store,runId:id,slot:2,report:raw,reportDigest:'d'.repeat(64)}),
      /Conflicting original Testing result archive/);
  }finally{await rm(root,{recursive:true,force:true});}
});
test('archive rejects incomplete false passes and corrupt committed records',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-testing-invalid-'));
  const data=path.join(root,'Data');await mkdir(data);
  const store=path.join(data,'Testing');
  try {
    await assert.rejects(()=>archiveTestingResult({
      store,runId:id,slot:2,report:{...original(),terminal:false},
      reportDigest:digest}));
    await archiveTestingResult({store,runId:id,slot:2,report:original(),
      reportDigest:digest});
    const file=path.join(store,'runs',id,'report.json');
    const before=await readFile(file,'utf8');
    await writeFile(file,before.replace('"NOT_TESTED"','"PASS"'));
    await assert.rejects(()=>readTestingArchive({store,runId:id}),
      /integrity mismatch/);
  }finally{await rm(root,{recursive:true,force:true});}
});
test('partial archives and unsafe parent links remain unavailable',async(t)=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-testing-partial-'));
  const data=path.join(root,'Data');await mkdir(data);
  const store=path.join(data,'Testing');
  try {
    await mkdir(store);await mkdir(path.join(store,'runs'));
    const run=path.join(store,'runs',id);
    await mkdir(run);await writeFile(path.join(run,'report.json'),
      JSON.stringify({schema:1})+'\n');
    await assert.rejects(()=>readTestingArchive({store,runId:id}));
    await assert.rejects(()=>archiveTestingResult({
      store,runId:id,slot:2,report:original(),reportDigest:digest}));
    const linked=path.join(root,'linked');await mkdir(linked);
    const linkedData=path.join(linked,'Data');
    try{await symlink(data,linkedData,'dir');}
    catch{return t.skip('No symlink support');}
    // The path spelling is valid Data/Testing, but Data is a link.
    await assert.rejects(()=>readTestingArchive({
      store:path.join(linkedData,'Testing'),runId:id}));
  }finally{await rm(root,{recursive:true,force:true});}
});
test('source-only blocked or incomplete report cannot counterfeit map PASS',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-testing-terminal-'));
  const data=path.join(root,'Data');await mkdir(data);
  try {
    const store=path.join(data,'Testing');
    const blocked=await archiveTestingResult({
      store,runId:id,slot:2,report:original('BLOCKED'),
      reportDigest:digest});
    assert.equal(blocked.reportedOutcome,'BLOCKED');
    assert.equal(blocked.reportedCounts,undefined);
    assert.equal(blocked.mapTesting,'NOT_TESTED');
    assert.equal(blocked.nativeRunState,'NOT_VERIFIED_AFTER_ARCHIVE_READ');
    await assert.rejects(()=>archiveTestingResult({
      store,runId:'b'.repeat(32),slot:2,
      report:{...original(),gameplayAcceptance:'PASS'},
      reportDigest:digest}));
  }finally{await rm(root,{recursive:true,force:true});}
});

test('archive slot inspection is read-only and blocks unsafe restart guesses',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-testing-scan-'));
  const data=path.join(root,'Data');await mkdir(data);
  const store=path.join(data,'Testing');
  try {
    assert.equal(await archivedTestingSlotReference({store,slot:2}),null);
    const saved=await archiveTestingResult({
      store,runId:id,slot:2,report:original(),reportDigest:digest});
    const bySlot=await archivedTestingSlotReference({store,slot:2});
    assert.equal(bySlot.runId,id);
    assert.equal(bySlot.slot,2);
    assert.equal(bySlot.recordedAt,saved.recordedAt);
    assert.equal(await archivedTestingSlotReference({store,slot:3}),null);
    const partial=path.join(store,'runs','b'.repeat(32));
    await mkdir(partial);
    await assert.rejects(()=>archivedTestingSlotReference({store,slot:2}),
      /ENOENT/);
    assert.equal(await readTestingArchive({store,runId:id}).reportedOutcome,
      'PROBE_REPORTED_PASS');
  }finally{await rm(root,{recursive:true,force:true});}
});
