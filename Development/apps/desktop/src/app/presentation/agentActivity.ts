// User-facing wording for the already-validated optional native status report.
// This never asserts Windows desktop control, actual input or map test PASS.
import type {AgentActivityReport,AgentActivityItem,AgentProbeCounts} from '../bridge/lazyTester.js';
export interface TestingSummaryView {
  slot:string;
  outcome:string;
  counts:AgentProbeCounts|null;
}
export interface ActivityView {
  summary:string;
  lines:string[];
  testingSummaries:TestingSummaryView[];
  urgent:boolean;
  stale:boolean;
}
const completeProbeOutcomes=new Set(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW']);
function isCompleteProbe(item:AgentActivityItem):boolean {
  return item.kind==='SKILLS_PROBE' &&
    item.status==='PROBE_REPORT_AVAILABLE' &&
    completeProbeOutcomes.has(item.reportedOutcome??'');
}
function reportedProbeLabel(outcome:string):string {
  switch(outcome) {
    case 'PROBE_REPORTED_PASS':return 'Reported pass';
    case 'FAIL':return 'Reported failures';
    case 'NEEDS_REVIEW':return 'Review required';
    default:return 'Unavailable';
  }
}
function line(item:AgentActivityItem):string {
  const slot='MC'+String(item.slot).padStart(2,'0');
  switch(item.status) {
    case 'NATIVE_RELEASE_UNCONFIRMED':return slot+' · Native AI input release unconfirmed';
    case 'STARTUP_RECONCILIATION_REQUIRED':return slot+' · AI input requires reconciliation';
    case 'EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT':return slot+' · Experimental input request (unverified)';
    case 'SAMPLER_ACTIVE_RECENT_FRAME':return slot+' · Local observation active';
    case 'SAMPLER_ACTIVE_NO_RECENT_FRAME':return slot+' · Observing; recent frame unavailable';
    case 'LOCAL_QA_PREPARATION_IN_PROGRESS':return slot+' · Preparing test setup';
    case 'LOCAL_QA_REVIEW_IN_PROGRESS':return slot+' · Reviewing test evidence';
    case 'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE':return slot+' · Testing setup ready; awaiting command or evidence';
    case 'CHECKPOINT_EXPIRED_OR_UNAVAILABLE':return slot+' · Testing status unavailable';
    case 'PROBE_REPORT_AVAILABLE':return slot+' · Probe result '+(
      item.reportedOutcome==='INCOMPLETE'?'incomplete':
      'blocked or unavailable');
    default:return slot+' · Status unavailable';
  }
}
export function presentAgentActivity(report:AgentActivityReport|null):ActivityView|null {
  if(!report||report.displayHint==='HIDDEN_NO_LOCAL_SIGNAL')return null;
  if(report.stale)return {
    summary:'AI & Testing · Status unavailable',
    lines:['An earlier local agent report needed attention, but its current status is unknown.'],
    testingSummaries:[],urgent:true,stale:true,
  };
  const sorted=[...report.items].sort((a,b)=>a.slot-b.slot);
  const urgent=report.displayHint==='ATTENTION';
  const completed=sorted.filter(isCompleteProbe);
  const testingSummaries=completed.map(item=>({
    slot:'MC'+String(item.slot).padStart(2,'0'),
    outcome:reportedProbeLabel(item.reportedOutcome??''),
    counts:item.probeCounts??null,
  }));
  // Avoid repeating the exact result as both a text line and a summary.
  const lines=sorted.filter(item=>!isCompleteProbe(item)).map(line);
  let summary=urgent?'AI & Testing · Needs attention':'AI & Testing · Activity';
  if(sorted.length===1) {
    const item=sorted[0];
    const slot='MC'+String(item.slot).padStart(2,'0');
    if(isCompleteProbe(item)) {
      summary=item.reportedOutcome==='PROBE_REPORTED_PASS'?
        'Testing · Results available · '+slot:
        item.reportedOutcome==='FAIL'?
          'Testing · Failures reported · '+slot:
          'Testing · Review needed · '+slot;
    } else if(item.kind==='SKILLS_PROBE'&&
        item.status==='PROBE_REPORT_AVAILABLE') {
      summary=item.reportedOutcome==='INCOMPLETE'?
        'Testing · Incomplete · '+slot:
        'Testing · Blocked · '+slot;
    } else if(!urgent) {
      summary=item.kind==='VISUAL_OBSERVATION'?'Observing · '+slot:
        item.kind==='AI_INPUT'?'AI input request · '+slot:
        item.status==='LOCAL_QA_PREPARATION_IN_PROGRESS'?
          'Testing · Preparing · '+slot:
        item.status==='LOCAL_QA_REVIEW_IN_PROGRESS'?
          'Testing · Reviewing · '+slot:
          'Testing · Waiting · '+slot;
    }
  }
  return {summary,lines,testingSummaries,urgent,stale:false};
}
