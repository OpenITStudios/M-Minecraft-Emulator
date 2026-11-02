// Source authored only. No test execution or real Minecraft/Windows access.
import test from 'node:test';
import assert from 'node:assert/strict';
import {projectAgentActivity} from './activity-status.mjs';

const at=Date.parse('2026-11-02T12:00:00.000Z');
const base=()=>({mode:'human-plus-four-ai',atMs:at});

test('idle/default MCP activity is hidden, never asserts no external PC control',()=>{
  const state=projectAgentActivity(base());
  assert.deepEqual(state.items,[]);
  assert.equal(state.displayHint,'HIDDEN_NO_LOCAL_SIGNAL');
  assert.equal(state.desktopControl,'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
  assert.equal(state.minecraftInputEffect,'NOT_VERIFIED');
});

test('local visual sampling does not establish AI input or video/game acceptance',()=>{
  const state=projectAgentActivity({...base(),visualSamplers:[
    {slot:4,status:'SAMPLER_ACTIVE_RECENT_FRAME'},
  ]});
  assert.equal(state.items.length,1);
  assert.deepEqual(state.items[0],{
    kind:'VISUAL_OBSERVATION',slot:4,
    status:'SAMPLER_ACTIVE_RECENT_FRAME',
    importance:'INFORMATION',evidence:'LOCAL_SAMPLER_ONLY',
  });
  assert.equal(state.desktopControl,'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
  assert.equal(state.minecraftInputEffect,'NOT_VERIFIED');
  assert.throws(()=>projectAgentActivity({...base(),visualSamplers:[
    {slot:1,status:'SAMPLER_ACTIVE_RECENT_FRAME'}]}),/Human MC01/);
});

test('opted-in native input startup hold is recovery, never a control-success claim',()=>{
  const state=projectAgentActivity({...base(),multiEnabled:true,
    multiSlots:[4,2],uncertainSlots:[4,3,2],startupReconcileSlots:[3]});
  assert.equal(state.displayHint,'ATTENTION');
  assert.deepEqual(state.items.map(v=>[v.slot,v.status]),[
    [2,'NATIVE_RELEASE_UNCONFIRMED'],
    [3,'STARTUP_RECONCILIATION_REQUIRED'],
    [4,'NATIVE_RELEASE_UNCONFIRMED'],
  ]);
  assert.equal(state.items.some(v=>v.status==='AI_CONTROLLING_PC'),false);
  assert.throws(()=>projectAgentActivity({...base(),multiEnabled:true,
    uncertainSlots:[1]}),/Human MC01/);
});

test('short input request remains transport only, not Minecraft gameplay',()=>{
  const state=projectAgentActivity({...base(),multiEnabled:true,multiSlots:[5,2]});
  assert.deepEqual(state.items.map(v=>v.slot),[2,5]);
  assert.ok(state.items.every(x=>x.status==='EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT'));
  assert.equal(state.minecraftInputEffect,'NOT_VERIFIED');
});

test('a prepared checkpoint does not imply command dispatch or QA execution',()=>{
  const state=projectAgentActivity({...base(),probePlans:[
    {slot:2,preparedAt:'2026-11-02T11:59:00.000Z',terminalStatus:null},
  ]});
  assert.equal(state.items[0].status,'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE');
  assert.equal(state.items[0].evidence,'SOURCE_CHECKPOINT_ONLY');
  assert.equal(state.mapGameplayAcceptance,'NOT_TESTED');
});

test('source reported PASS is not map PASS; stopped/expired are truthful',()=>{
  const reported=projectAgentActivity({...base(),probePlans:[
    {slot:2,preparedAt:'2026-11-02T11:30:00.000Z',terminalStatus:'PROBE_REPORTED_PASS'},
  ]});
  assert.equal(reported.items[0].status,'PROBE_REPORT_AVAILABLE');
  assert.equal(reported.items[0].reportedOutcome,'PROBE_REPORTED_PASS');
  assert.equal(reported.mapGameplayAcceptance,'NOT_TESTED');
  assert.equal(reported.items[0].importance,'INFORMATION');
  const expired=projectAgentActivity({...base(),probePlans:[
    {slot:4,preparedAt:'2026-11-02T11:30:00.000Z',terminalStatus:null},
  ]});
  assert.equal(expired.items[0].status,'CHECKPOINT_EXPIRED_OR_UNAVAILABLE');
  assert.equal(expired.displayHint,'ATTENTION');
  const missing=projectAgentActivity({...base(),probeBusy:[3]});
  assert.equal(missing.items[0].status,'LOCAL_QA_PREPARATION_IN_PROGRESS');
});

test('malformed/caller-provided claims cannot mark desktop controlled',()=>{
  assert.throws(()=>projectAgentActivity({...base(),probePlans:[
    {slot:2,terminalStatus:'PROBE_REPORTED_PASS'},
    {slot:2,terminalStatus:'FAIL'},
  ]}),/Invalid Probe activity/);
  assert.throws(()=>projectAgentActivity({...base(),visualSamplers:[
    {slot:4,status:'AI_CONTROLLING_PC'},
  ]}),/Invalid sampler activity/);
});

test('original Probe summary carries only complete exact counts, not map acceptance',()=>{
  const input={...base(),probePlans:[{slot:2,
    terminalStatus:'PROBE_REPORTED_PASS',
    expectedCount:3,terminalCounts:{pass:3,fail:0,info:0,look:0},
  }]};
  const view=projectAgentActivity(input);
  assert.deepEqual(view.items[0].probeCounts,
    {expected:3,pass:3,fail:0,info:0,look:0});
  assert.equal(view.mapGameplayAcceptance,'NOT_TESTED');
  const unknown=projectAgentActivity({...base(),probePlans:[{slot:2,
    terminalStatus:'PROBE_REPORTED_PASS',expectedCount:3,
    terminalCounts:{pass:4,fail:0,info:0,look:0},
  }]});
  assert.equal(Object.hasOwn(unknown.items[0],'probeCounts'),false,
    'Mismatched totals cannot reach the results panel');
  for(const status of ['BLOCKED','INCOMPLETE']) {
    const unverified=projectAgentActivity({...base(),probePlans:[{slot:2,
      terminalStatus:status,expectedCount:3,
      terminalCounts:{pass:3,fail:0,info:0,look:0},
    }]});
    assert.equal(Object.hasOwn(unverified.items[0],'probeCounts'),false,
      'Blocked or incomplete reports never display complete-run counts');
  }
});

test('Probe failure and review counters require matching original outcomes',()=>{
  const failed=projectAgentActivity({...base(),probePlans:[{slot:4,
    terminalStatus:'FAIL',expectedCount:2,
    terminalCounts:{pass:1,fail:1,info:0,look:0},
  }]});
  assert.deepEqual(failed.items[0].probeCounts,
    {expected:2,pass:1,fail:1,info:0,look:0});
  assert.equal(failed.displayHint,'ATTENTION');
  const review=projectAgentActivity({...base(),probePlans:[{slot:4,
    terminalStatus:'NEEDS_REVIEW',expectedCount:2,
    terminalCounts:{pass:1,fail:0,info:1,look:0},
  }]});
  assert.equal(review.items[0].probeCounts.info,1);
  const invalid=projectAgentActivity({...base(),probePlans:[{slot:4,
    terminalStatus:'NEEDS_REVIEW',expectedCount:2,
    terminalCounts:{pass:2,fail:0,info:0,look:0},
  }]});
  assert.equal(Object.hasOwn(invalid.items[0],'probeCounts'),false);
});
