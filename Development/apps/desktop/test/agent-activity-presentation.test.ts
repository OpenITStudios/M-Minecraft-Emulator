import {describe,expect,it} from 'vitest';
import {parseAgentActivity,type AgentActivityItem} from '../src/app/bridge/lazyTester.js';
import {presentAgentActivity} from '../src/app/presentation/agentActivity.js';
import {sameAgentObservation} from '../src/app/inspectionScheduling.js';

const routine:AgentActivityItem={kind:'VISUAL_OBSERVATION',slot:1,status:'SAMPLER_ACTIVE_RECENT_FRAME',importance:'INFORMATION',evidence:'LOCAL_SAMPLER_ONLY'};
const waiting:AgentActivityItem={kind:'SKILLS_PROBE',slot:2,status:'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE',importance:'INFORMATION',evidence:'SOURCE_CHECKPOINT_ONLY'};
const result:AgentActivityItem={kind:'SKILLS_PROBE',slot:2,status:'PROBE_REPORT_AVAILABLE',importance:'INFORMATION',evidence:'ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN',reportedOutcome:'PROBE_REPORTED_PASS'};
const attention:AgentActivityItem={kind:'AI_INPUT',slot:3,status:'NATIVE_RELEASE_UNCONFIRMED',importance:'ATTENTION',evidence:'LOCAL_MCP_RECOVERY_HOLD'};
const report=(items:AgentActivityItem[],stale=false)=>parseAgentActivity({
  schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',snapshotAt:'2026-11-26T12:00:00.000Z',
  displayHint:items.some(i=>i.importance==='ATTENTION')?'ATTENTION':items.length?'ACTIVITY':'HIDDEN_NO_LOCAL_SIGNAL',
  desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',minecraftInputEffect:'NOT_VERIFIED',
  mapGameplayAcceptance:'NOT_TESTED',items,stale,
});
describe('Contextual AI and Testing disclosure',()=>{
  it('hides routine observation and waiting',()=>{
    expect(presentAgentActivity(report([routine,waiting]))).toBeNull();
  });
  it('shows terminal results without routine lines',()=>{
    const view=presentAgentActivity(report([routine,result]));
    expect(view?.summary).toBe('Testing · Results available · MC02');
    expect(view?.lines).toEqual([]);
    expect(view?.testingSummaries[0]?.outcome).toBe('Reported pass');
  });
  it('retains recovery attention and stale warnings',()=>{
    expect(presentAgentActivity(report([routine,attention]))?.lines).toEqual(['MC03 · Native AI input release unconfirmed']);
    expect(presentAgentActivity(report([attention],true))?.stale).toBe(true);
  });
});

describe('Agent observation equality',()=>{
  const counted:AgentActivityItem={...result,reportedOutcome:'NEEDS_REVIEW',importance:'ATTENTION',
    probeCounts:{expected:4,pass:1,fail:0,info:1,look:2}};
  const baseline=report([routine,counted]);
  it('ignores property order but keeps item order and freshness',()=>{
    const reordered=Object.fromEntries(Object.entries(baseline).reverse());
    reordered.items=[routine,Object.fromEntries(Object.entries(counted).reverse())];
    expect(sameAgentObservation(baseline,parseAgentActivity(reordered))).toBe(true);
    expect(sameAgentObservation(baseline,report([counted,routine]))).toBe(false);
    expect(sameAgentObservation(baseline,report([routine,counted],true))).toBe(false);
    expect(sameAgentObservation(baseline,parseAgentActivity({...baseline,snapshotAt:'2026-11-26T12:00:01.000Z'}))).toBe(false);
  });
  it('retains optional Probe receipts and all count fields',()=>{
    expect(sameAgentObservation(baseline,report([routine,{...counted,probeCounts:{...counted.probeCounts!}}]))).toBe(true);
    const {probeCounts: _counts,...withoutCounts}=counted;
    expect(sameAgentObservation(baseline,report([routine,withoutCounts]))).toBe(false);
    for(const probeCounts of [
      {expected:5,pass:1,fail:0,info:1,look:3},
      {expected:4,pass:2,fail:0,info:1,look:1},
      {expected:4,pass:1,fail:0,info:2,look:1},
    ]) expect(sameAgentObservation(baseline,report([routine,{...counted,probeCounts}]))).toBe(false);
    expect(sameAgentObservation(baseline,report([routine,{...counted,reportedOutcome:'FAIL',
      probeCounts:{expected:4,pass:0,fail:1,info:1,look:2}}]))).toBe(false);
    expect(sameAgentObservation(baseline,report([{...routine,slot:4},counted]))).toBe(false);
    expect(sameAgentObservation(baseline,report([{...routine,status:'SAMPLER_ACTIVE_NO_RECENT_FRAME'},counted]))).toBe(false);
  });
});
