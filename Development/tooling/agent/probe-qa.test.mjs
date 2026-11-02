// SOURCE-AUTHORED ONLY. Run separately when source checks are approved.
import test from 'node:test';
import assert from 'node:assert/strict';
import {parseProbeCapturedLine,reviewProbeRun} from './probe-qa.mjs';

const stamp='2026-11-02T07:00:00.000Z';
const line=(issue,fields,time=stamp)=>
  time+' warn [probe:probe] INFO tick=120 issue='+issue+' '+fields;
const session={preparedAt:'2026-11-02T06:59:59.000Z',autoIds:['P01','P02']};
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
  assert.equal(parseProbeCapturedLine(line('RUN_DONE','arg=all count=1000')),null);
  const escaped=JSON.stringify('[probe:probe] INFO tick=121 issue=RESULT id=P01 result=pass build=dev detail=x').slice(1,-1);
  assert.equal(parseProbeCapturedLine(stamp+' warn '+escaped).id,'P01');
});

test('old stored or earlier run cannot become fresh PASS',()=>{
  const earlier=markers('2026-11-02T06:58:59.000Z');
  assert.equal(reviewProbeRun(earlier,session).status,'WAITING_FOR_NEW_RUN_START');
  const stored=parseProbeCapturedLine(line('STORED','id=P01 result=pass build=dev'));
  assert.equal(reviewProbeRun([stored],session).status,'WAITING_FOR_NEW_RUN_START');
});

test('only one complete run with exactly expected results is reported',()=>{
  const passed=reviewProbeRun(markers(),session);
  assert.equal(passed.status,'PROBE_REPORTED_ALL_PASS');
  assert.equal(passed.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(passed.counts.pass,2);
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
  assert.equal(reviewProbeRun(overlap,session).status,'INCOMPLETE_OVERLAPPING_RUNS');
  assert.equal(reviewProbeRun(markers().slice(0,3),session).status,'WAITING_FOR_RUN_DONE');
});

test('missing, absent or unusable local capture never implies game success',()=>{
  assert.equal(reviewProbeRun([],session,{captureStatus:'NO_CAPTURE_FILE'}).status,'NO_CAPTURE_FILE');
  assert.equal(reviewProbeRun([],null).status,'NOT_PREPARED');
});
