// User-facing wording for the already-validated optional native status report.
// This never asserts Windows desktop control, actual input or map test PASS.
import type {AgentActivityReport,AgentActivityItem} from '../bridge/lazyTester.js';
export interface ActivityView {summary:string;lines:string[];urgent:boolean;stale:boolean}
function line(item:AgentActivityItem):string {
  const slot='MC'+String(item.slot).padStart(2,'0');
  switch(item.status) {
    case 'NATIVE_RELEASE_UNCONFIRMED':return slot+' · Native AI input release unconfirmed';
    case 'STARTUP_RECONCILIATION_REQUIRED':return slot+' · AI input requires reconciliation';
    case 'EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT':return slot+' · Experimental input request (unverified)';
    case 'SAMPLER_ACTIVE_RECENT_FRAME':return slot+' · Local observation active';
    case 'SAMPLER_ACTIVE_NO_RECENT_FRAME':return slot+' · Observing; recent frame unavailable';
    case 'LOCAL_QA_PREPARATION_IN_PROGRESS':return slot+' · Preparing QA source';
    case 'LOCAL_QA_REVIEW_IN_PROGRESS':return slot+' · Reviewing Probe evidence';
    case 'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE':return slot+' · QA prepared; command/evidence pending';
    case 'CHECKPOINT_EXPIRED_OR_UNAVAILABLE':return slot+' · Probe checkpoint unavailable';
    case 'PROBE_REPORT_AVAILABLE':
      return slot+' · Original Probe result: '+(
        item.reportedOutcome==='PROBE_REPORTED_PASS'?'reported pass (map not tested)':
        item.reportedOutcome==='FAIL'?'fail reported':
        item.reportedOutcome==='NEEDS_REVIEW'?'needs review':
        item.reportedOutcome==='INCOMPLETE'?'incomplete':
        'blocked or unverified');
    default:return slot+' · Status unavailable';
  }
}
export function presentAgentActivity(report:AgentActivityReport|null):ActivityView|null {
  if(!report||report.displayHint==='HIDDEN_NO_LOCAL_SIGNAL')return null;
  if(report.stale)return {
    summary:'AI/QA status unavailable',
    lines:['An earlier local agent report needed attention, but its current status is unknown.'],
    urgent:true,stale:true,
  };
  const sorted=[...report.items].sort((a,b)=>a.slot-b.slot);
  const urgent=report.displayHint==='ATTENTION';
  let summary=urgent?'AI/QA needs attention':'AI/QA activity';
  if(sorted.length===1&&!urgent) {
    const item=sorted[0];
    const slot='MC'+String(item.slot).padStart(2,'0');
    summary=item.kind==='VISUAL_OBSERVATION'?'Observing · '+slot:
      item.kind==='AI_INPUT'?'Input request · '+slot:
      item.status==='PROBE_REPORT_AVAILABLE'?'Probe result · '+slot:
      'QA prepared · '+slot;
  }
  return {summary,lines:sorted.map(line),urgent,stale:false};
}
