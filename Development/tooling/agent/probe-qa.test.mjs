// SOURCE-AUTHORED ONLY. Run separately when source checks are approved.
import test from 'node:test';
import assert from 'node:assert/strict';
import {parseProbeCapturedLine,reviewProbeRun,compactProbeQaResult,
  probeQaProofGate,MAX_AUTO_PROBES} from './probe-qa.mjs';

const stamp='2026-11-02T07:00:00.000Z';
const line=(issue,fields,time=stamp)=>
  time+' warn [probe:probe] INFO tick=120 issue='+issue+' '+fields;
const session={preparedAt:'2026-11-02T06:59:59.000Z',autoIds:['P01','P02'],
  captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
const markers=(first='2026-11-02T07:00:00.000Z')=>[
  parseProbeCapturedLine(line('RUN_START','arg=auto count=2',first)),
  parseProbeCapturedLine(line('RESULT','id=P01 result=pass build=dev detail=okay',
    '2026-11-02T07:00:01.000Z')),
  parseProbeCapturedLine(line('RESULT','id=P02 result=pass build=dev detail=okay',
    '2026-11-02T07:00:02.000Z')),
  parseProbeCapturedLine(line('RUN_DONE','arg=auto count=2',
    '2026-11-02T07:00:03.000Z')),
];

test('probe marker parser accepts only canonical runner markers',()=>{
  assert.deepEqual(parseProbeCapturedLine(line('RUN_START','arg=auto count=2')).kind,'start');
  assert.equal(parseProbeCapturedLine(line('RESULT','id=P01 result=look build=dev detail=anything')).result,'look');
  assert.equal(parseProbeCapturedLine(line('STORED','id=P01 result=pass build=dev')),null);
  assert.equal(parseProbeCapturedLine(stamp+' log Minecraft says issue=RUN_DONE arg=auto count=2'),null);
  assert.equal(parseProbeCapturedLine(line('RESULT','id=P01 result=excellent build=dev')),null);
  const raw='[probe:probe] INFO tick=261598 issue=RESULT id=P01 result=pass build=dev detail=okay';
  const parsedRaw=parseProbeCapturedLine(raw);
  assert.equal(parsedRaw.id,'P01');
  assert.equal(parsedRaw.at,null);
  assert.equal(parsedRaw.tick,261598);
  const suppressed=parseProbeCapturedLine(
    '[probe:probe] INFO tick=261599 issue=RUN_START arg=auto count=2 (suppressed 1)');
  assert.equal(suppressed.throttled,true);
  assert.equal(parseProbeCapturedLine(line('RUN_DONE','arg=all count=1000')),null);
  const escaped=JSON.stringify('[probe:probe] INFO tick=121 issue=RESULT id=P01 result=pass build=dev detail=x').slice(1,-1);
  assert.equal(parseProbeCapturedLine(stamp+' warn '+escaped).id,'P01');
});

test('old STORED results cannot be classified as an appended fresh run',()=>{
  const stored=parseProbeCapturedLine(line('STORED','id=P01 result=pass build=dev'));
  assert.equal(reviewProbeRun([stored].filter(Boolean),session).status,'WAITING_FOR_NEW_RUN_START');
  assert.equal(reviewProbeRun([],session).status,'WAITING_FOR_NEW_RUN_START');
  // Source timestamps are not the freshness authority. A stable-file cursor
  // from the local capture reader defines what was appended after prepare.
  assert.equal(reviewProbeRun(markers('2026-11-02T06:58:59.000Z'),session)
    .evidenceBoundary,'SAME_STABLE_FILE_APPENDED_BYTES_AFTER_PREPARE');
});

test('only one complete run with exactly expected results is reported',()=>{
  const passed=reviewProbeRun(markers(),session);
  assert.equal(passed.status,'PROBE_REPORTED_ALL_PASS');
  assert.equal(passed.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(passed.counts.pass,2);
  assert.equal(passed.fromTick,120);
  assert.equal(passed.toTick,120);
  const missing=reviewProbeRun([markers()[0],markers()[1],markers()[3]],session);
  assert.equal(missing.status,'INCOMPLETE_RESULTS');
  assert.deepEqual(missing.missingIds,['P02']);
  const wrong=reviewProbeRun(markers().map((v,i)=>i===0?{...v,count:12}:v),session);
  assert.equal(wrong.status,'INCOMPLETE_RESULTS');
});

test('fail and look stay distinct; overlapping runs are incomplete',()=>{
  const bad=markers();
  bad[2]={...bad[2],result:'fail'};
  assert.equal(reviewProbeRun(bad,session).probeOutcome,'FAIL');
  bad[2]={...bad[2],result:'look'};
  assert.equal(reviewProbeRun(bad,session).status,'PROBE_REVIEW_REQUIRED');
  const overlap=[markers()[0],markers()[0],...markers().slice(1)];
  assert.equal(reviewProbeRun(overlap,session).status,'INCOMPLETE_UNEXPECTED_RUN');
  assert.equal(reviewProbeRun(markers().slice(0,3),session).status,'WAITING_FOR_RUN_DONE');
});

test('missing, absent or unusable local capture never implies game success',()=>{
  assert.equal(reviewProbeRun([],session,{captureStatus:'NO_CAPTURE_FILE'}).status,'NO_CAPTURE_FILE');
  assert.equal(reviewProbeRun([],null).status,'NOT_PREPARED');
});

test('unrelated runs, repeated records and tick resets cannot be PASS',()=>{
  const full=markers();
  const duplicated=[full[0],full[1],full[1],full[2],full[3]];
  assert.equal(reviewProbeRun(duplicated,session).status,'INCOMPLETE_RESULTS');
  const other=[{...full[0],arg:'all'},...full.slice(1)];
  assert.equal(reviewProbeRun(other,session).status,'INCOMPLETE_UNEXPECTED_RUN');
  const noStart=[...full.slice(1)];
  assert.equal(reviewProbeRun(noStart,session).status,'INCOMPLETE_NO_RUN_START');
  const restarted=[full[0],full[1],{...full[2],tick:80},full[3]];
  assert.equal(reviewProbeRun(restarted,session).status,'INCOMPLETE_PROBE_TICK_REGRESSION');
});

test('capture checkpoint mismatch remains unavailable, not a test failure',()=>{
  const result=reviewProbeRun(markers(),session,{
    captureStatus:'CAPTURE_FILE_UNSTABLE',
    captureReason:'PROBE_CAPTURE_REPLACED'});
  assert.equal(result.status,'CAPTURE_FILE_UNSTABLE');
  assert.equal(result.probeOutcome,'NOT_ESTABLISHED');
  assert.equal(result.captureIssue,'PROBE_CAPTURE_REPLACED');
});

test('logger throttle notices block acceptance even with complete result counts',()=>{
  const flagged=markers().map((e,i)=>i===0?{...e,throttled:true}:e);
  assert.equal(reviewProbeRun(flagged,session).status,'INCOMPLETE_THROTTLED_MARKERS');
  assert.equal(reviewProbeRun(flagged,session).gameplayAcceptance,'NOT_VERIFIED');
});


test('compact one-call AI QA summary keeps Probe status separate from gameplay',()=>{
  const approved=compactProbeQaResult({
    status:'PROBE_REPORTED_ALL_PASS',expectedCount:2,
    counts:{pass:2,fail:0,info:0,look:0},missingIds:[],
    gameplayAcceptance:'NOT_VERIFIED'},2);
  assert.equal(approved.status,'PROBE_REPORTED_PASS');
  assert.equal(approved.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(approved.commandSentByLazyTester,undefined);
  assert.equal(approved.resultOrigin,undefined);
  assert.equal(approved.reason,undefined);
  assert.equal(approved.counts.pass,2);
  const failed=compactProbeQaResult({
    status:'PROBE_REPORTED_FAILURES',counts:{pass:1,fail:1,info:0,look:0}},2);
  assert.equal(failed.status,'FAIL');
  const review=compactProbeQaResult({
    status:'PROBE_REVIEW_REQUIRED',counts:{pass:1,fail:0,info:0,look:1}},2);
  assert.equal(review.status,'NEEDS_REVIEW');
  const pending=compactProbeQaResult({status:'WAITING_FOR_RUN_DONE'},2);
  assert.equal(pending.status,'WAITING');
  assert.equal(pending.reason,'WAITING_FOR_RUN_DONE');
  const incomplete=compactProbeQaResult({
    status:'CAPTURE_FILE_UNSTABLE',captureIssue:'PROBE_CAPTURE_REPLACED',
    missingIds:['P01','P02']},2);
  assert.equal(incomplete.status,'INCOMPLETE');
  assert.equal(incomplete.captureIssue,'PROBE_CAPTURE_REPLACED');
  assert.equal(compactProbeQaResult({status:'QA_PLAN_EXPIRED'},2).status,'BLOCKED');
  assert.deepEqual(compactProbeQaResult({
    status:'INCOMPLETE_RESULTS',
    missingIds:Array.from({length:30},(_,i)=>'P'+i)},2).missingIds,
    ['P0','P1','P2','P3','P4','P5','P6','P7']);
  assert.equal(compactProbeQaResult({
    status:'PROBE_REPORTED_ALL_PASS',counts:{
      pass:2,fail:0,info:0,look:-1}},2).counts,undefined);
});

test('proof gate never authorizes dispatch or certifies world/Player from file and window',()=>{
  const strong=probeQaProofGate({sourceReady:true,captureReady:true,targetBound:true});
  assert.equal(strong.readiness,'BLOCKED');
  assert.equal(strong.automaticDispatch,'DISABLED');
  assert.equal(strong.logOrigin,'NOT_VERIFIED');
  assert.equal(strong.qaWorld,'NOT_VERIFIED');
  assert.equal(strong.playerAdmin,'NOT_VERIFIED');
  assert.deepEqual(strong.missingEvidence,[
    'AUTHORIZED_PLAYER_ADMIN_COMMAND_DELIVERY',
    'EXACT_QA_WORLD_IDENTITY',
    'SOURCE_BOUND_CONTENT_LOG_ORIGIN',
  ]);
  const unbound=probeQaProofGate({sourceReady:true,captureReady:true});
  assert.equal(unbound.targetAtPrepare,'NOT_VERIFIED');
  assert.equal(unbound.missingEvidence.includes('VERIFIED_NATIVE_TARGET_AT_PREPARE'),true);
  const absent=probeQaProofGate();
  assert.equal(absent.missingEvidence.includes('APPROVED_PROBE_SOURCE'),true);
  assert.equal(absent.missingEvidence.includes('STABLE_CAPTURE_CHECKPOINT'),true);
  assert.throws(()=>probeQaProofGate({sourceReady:'yes'}),TypeError);
});

test('mixed build provenance is incomplete even if all RESULT IDs pass',()=>{
  const events=markers();
  events[2]={...events[2],build:'release'};
  const mixed=reviewProbeRun(events,session);
  assert.equal(mixed.status,'INCOMPLETE_MIXED_PROBE_BUILDS');
  assert.equal(mixed.probeOutcome,'NOT_ESTABLISHED');
  assert.equal(mixed.gameplayAcceptance,'NOT_VERIFIED');
  const valid=reviewProbeRun(markers(),session);
  assert.equal(valid.reportedBuild,'dev');
  assert.equal(compactProbeQaResult(valid,2).reportedBuild,'dev');
});

test('official Probe auto-catalog bound is exported for the map report consumer',()=>{
  assert.equal(MAX_AUTO_PROBES,80);
});
