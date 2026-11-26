import {describe,expect,it} from 'vitest';
import {parseAgentActivity,type AgentActivityItem} from '../src/app/bridge/lazyTester.js';
import {presentAgentActivity} from '../src/app/presentation/agentActivity.js';

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
