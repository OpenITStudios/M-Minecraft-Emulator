// Source-authored fixtures only. Do not execute without separate approval.
import test from 'node:test';
import assert from 'node:assert/strict';
import {summarizeEducationCapture} from './education-log-probe.mjs';

const run='edu-script-probe';
const sample=(seq,tick,phase=seq===1?'START':'DEFERRED')=>
  '[LT_EDU_PROBE]'+JSON.stringify({
    schema:1,run,seq,tick,kind:'script_probe',phase,
  });
const captured=(seq,tick)=>'2026-11-02T00:30:01.000Z warn '+
  sample(seq,tick)+'\n';

test('recognizes exact deliberate Script API probe markers, never gameplay PASS',()=>{
  const result=summarizeEducationCapture(
    sample(1,10)+'\n'+sample(2,30)+'\n',
    {run,format:'marker-lines'});
  assert.equal(result.status,'CANDIDATE_MARKER_PAIR_FOUND');
  assert.equal(result.validMarkerCount,2);
  assert.equal(result.candidateTickProgress,20);
  assert.equal(result.minecraftEducationTransport,'NOT_VERIFIED');
  assert.equal(result.gameplay,'NOT_VERIFIED');
  assert.equal(result.rawLogSharedWithAI,false);
  assert.equal(JSON.stringify(result).includes('account'),false);
});

test('matches explicitly selected timestamped PrintEvent-like wrapper',()=>{
  const output=summarizeEducationCapture(
    '2026-11-02T00:30:01.000Z log ordinary text\n'+
    captured(1,17)+captured(2,37),
    {run,format:'timestamped-print'});
  assert.equal(output.linesInspected,3);
  assert.equal(output.validMarkerCount,2);
  assert.equal(output.unrecognizedLines,1);
  assert.equal(output.status,'CANDIDATE_MARKER_PAIR_FOUND');
});

test('supports exactly anchored escaped PrintEvent JSON strings',()=>{
  const esc=(seq,tick)=>sample(seq,tick).replaceAll('"','\\"');
  const content='2026-11-02T00:30:01.000Z warn '+esc(1,11)+'\n'+
    '2026-11-02T00:30:02.000Z warn '+esc(2,31)+'\n';
  const result=summarizeEducationCapture(content,{
    run,format:'timestamped-print'});
  assert.equal(result.status,'CANDIDATE_MARKER_PAIR_FOUND');
  assert.equal(result.validMarkerCount,2);
});

test('does not treat embedded, changed-run, malformed or secret fields as trusted markers',()=>{
  const untrusted=[
    'chat [LT_EDU_PROBE]'+JSON.stringify({
      schema:1,run,seq:1,tick:10,kind:'script_probe',phase:'START'}),
    '[LT_EDU_PROBE]{not json}',
    '[LT_EDU_PROBE]'+JSON.stringify({
      schema:1,run,seq:1,tick:10,kind:'script_probe',phase:'START',
      player:'private-name',
    }),
    sample(1,10).replace(run,'wrong-run'),
    sample(1,10).replace('"START"','"DEFERRED"'),
  ].join('\n')+'\n';
  const output=summarizeEducationCapture(untrusted,{run,format:'marker-lines'});
  assert.equal(output.status,'REJECTED_MARKERS');
  assert.equal(output.markerCandidates,4);
  assert.equal(output.invalidMarkers,4);
  assert.equal(JSON.stringify(output).includes('private-name'),false);
  assert.equal(JSON.stringify(output).includes('wrong-run'),false);
});

test('missing, duplicate, unordered or non-progressing marker pairs cannot pass',()=>{
  for(const content of [
    sample(1,10)+'\n',
    sample(1,10)+'\n'+sample(1,20)+'\n'+sample(2,30)+'\n',
    sample(2,30)+'\n'+sample(1,10)+'\n',
    sample(1,10)+'\n'+sample(2,10)+'\n',
  ]) {
    const r=summarizeEducationCapture(content,{run,format:'marker-lines'});
    if(content.includes(sample(2,30))&&content.startsWith(sample(2,30))) {
      // Source order is not verified by "counts"; bad producer sequence
      // must remain a partial capture rather than a complete pair.
      assert.notEqual(r.status,'CANDIDATE_MARKER_PAIR_FOUND');
    }else{
      assert.notEqual(r.status,'CANDIDATE_MARKER_PAIR_FOUND');
    }
  }
});

test('invalid modes, IDs, unfinished and oversized captures are refused',()=>{
  assert.throws(()=>summarizeEducationCapture(sample(1,10),{
    run,format:'marker-lines'}));
  assert.throws(()=>summarizeEducationCapture('',{run:'../../secret',
    format:'marker-lines'}));
  assert.throws(()=>summarizeEducationCapture('',{run,format:'auto'}));
  assert.throws(()=>summarizeEducationCapture(
    sample(1,10)+'\n'.repeat(12002),{run,format:'marker-lines'}));
  const oversized='[LT_EDU_PROBE]'+('A'.repeat(4097))+'\n';
  const summary=summarizeEducationCapture(oversized,{run,format:'marker-lines'});
  assert.equal(summary.overlongLines,1);
  assert.notEqual(summary.status,'CANDIDATE_MARKER_PAIR_FOUND');
});

test('unrecognized captured log lines never leak raw message contents',()=>{
  const source='2026-11-02T00:31:00.000Z error token=sensitive@example.org\n'+
    '2026-11-02T00:31:01.000Z warn [LT_EDU_PROBE]{bad json}\n';
  const output=summarizeEducationCapture(source,{
    run,format:'timestamped-print'});
  assert.equal(output.status,'REJECTED_MARKERS');
  assert.equal(output.invalidMarkers,1);
  const encoded=JSON.stringify(output);
  assert.equal(encoded.includes('sensitive@example.org'),false);
  assert.equal(encoded.includes('token='),false);
});
