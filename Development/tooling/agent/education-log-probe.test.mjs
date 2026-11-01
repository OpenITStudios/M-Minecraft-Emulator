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
  assert.equal(output.status,'UNSUPPORTED_MARKER_WRAPPER_OR_CONTEXT');
  assert.equal(output.unparsedMarkerLines,1);
  assert.equal(output.markerCandidates,4);
  assert.equal(output.invalidMarkers,4);
  assert.equal(JSON.stringify(output).includes('private-name'),false);
  assert.equal(JSON.stringify(output).includes('wrong-run'),false);
});

test('survey classifies wrapper shapes without parsing or revealing marker payloads',()=>{
  const secret='private path C:\\Users\\Hidden\\minecraft token=example';
  const input=[
    'routine console text',
    sample(1,10),
    captured(2,30).trimEnd(),
    secret+' '+sample(1,50),
    'private chat '+sample(1,80)+' '+sample(2,100),
  ].join('\n')+'\n';
  const output=summarizeEducationCapture(input,{
    run,format:'envelope-survey'});
  assert.equal(output.status,'ENVELOPE_SURVEY_ONLY_NO_TRANSPORT_PROOF');
  assert.equal(output.linesInspected,5);
  assert.equal(output.markerBearingLines,4);
  assert.equal(output.envelopeCounts.direct,1);
  assert.equal(output.envelopeCounts.timestamped,1);
  assert.equal(output.envelopeCounts.otherPrefix,2);
  assert.equal(output.envelopeCounts.repeatedMarkerLines,1);
  assert.equal(output.markerPayloadsRead,false);
  assert.equal(output.minecraftEducationTransport,'NOT_VERIFIED');
  assert.equal(output.rawLogSharedWithAI,false);
  assert.equal(JSON.stringify(output).includes(secret),false);
  assert.equal(JSON.stringify(output).includes('C:\\Users'),false);
  assert.equal(JSON.stringify(output).includes('token='),false);
});

test('unknown Education wrapper is not incorrectly reported as absent output',()=>{
  const unknown='[EDUCATION-UNVERIFIED] ';
  const capturedLines=unknown+sample(1,100)+'\n'+
    unknown+sample(2,120)+'\n';
  for(const format of ['marker-lines','timestamped-print']) {
    const out=summarizeEducationCapture(capturedLines,{run,format});
    assert.equal(out.status,'UNSUPPORTED_MARKER_WRAPPER_OR_CONTEXT');
    assert.equal(out.unparsedMarkerLines,2);
    assert.equal(out.validMarkerCount,0);
    assert.equal(out.candidateTickProgress,null);
    assert.equal(out.gameplay,'NOT_VERIFIED');
  }
  const survey=summarizeEducationCapture(capturedLines,{
    run,format:'envelope-survey'});
  assert.equal(survey.envelopeCounts.otherPrefix,2);
  assert.equal(survey.markerPayloadsRead,false);
});

test('extra marker-bearing chat text prevents a false positive marker pair',()=>{
  const content=sample(1,40)+'\n'+sample(2,60)+'\n'+
    'chat: '+sample(1,70)+'\n';
  const output=summarizeEducationCapture(content,{
    run,format:'marker-lines'});
  assert.equal(output.validMarkerCount,2);
  assert.equal(output.unparsedMarkerLines,1);
  assert.equal(output.status,'UNSUPPORTED_MARKER_WRAPPER_OR_CONTEXT');
  assert.equal(output.candidateTickProgress,null);
});

test('survey counts oversized marker-bearing lines but never returns their content',()=>{
  const secret='token=hiddenvalue';
  const input=secret+' '+('x'.repeat(4100))+sample(1,10)+'\n';
  const out=summarizeEducationCapture(input,{
    run,format:'envelope-survey'});
  assert.equal(out.overlongLines,1);
  assert.equal(out.overlongMarkerLines,1);
  assert.equal(out.markerBearingLines,0);
  assert.equal(JSON.stringify(out).includes(secret),false);
  assert.equal(out.status,'ENVELOPE_SURVEY_ONLY_NO_TRANSPORT_PROOF');
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
