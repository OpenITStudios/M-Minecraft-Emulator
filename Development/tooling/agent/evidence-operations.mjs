// Bounded read-only evidence projection and explicit per-slot capture baselines.
// No background collector, native authority, or second Probe checkpoint.
import {slotOf,keysExactly} from './agent-contract.mjs';
import {EVIDENCE_LIMIT,validCaptureCursor} from './evidence-reader.mjs';
import {summarizeGameEvidence,summarizeEvidenceReports,normalizeEvidenceSlots} from './evidence-summary.mjs';

export function createEvidenceOperations(config,{evidence,requireOpen}) {
  // At most one local evidence baseline per slot, separate from Probe QA.
  // No persistent run store, player attribution, or background collector.
  const evidenceBaselines=new Map();
  let evidenceWindowBusy=false;
  async function summarize(args) {
      keysExactly(args,['slots','window']);
      if(args.window===undefined) {
        return summarizeGameEvidence(config.evidenceDirectory,
          args.slots ?? [1,2,3,4,5],evidence);
      }
      if(!['begin','read'].includes(args.window)||!Object.hasOwn(args,'slots')) {
        throw new Error('Invalid tool arguments');
      }
      const slots=normalizeEvidenceSlots(args.slots);
      if(evidenceWindowBusy)throw new Error('Evidence window operation in progress');
      evidenceWindowBusy=true;
      try {
        const begin=args.window==='begin';
        // A failed explicit new baseline must not reuse an earlier test's
        // cursor. Repeated read stays anchored; no implicit rebase or retry.
        if(begin)for(const slot of slots)evidenceBaselines.delete(slot);
        const reports=await Promise.all(slots.map(async slot=>{
          const checkpoint=evidenceBaselines.get(slot);
          if(!begin&&!checkpoint) {
            return {slot,status:'UNAVAILABLE',events:[],baselineRequired:true};
          }
          try {
            return await evidence(config.evidenceDirectory,slot,EVIDENCE_LIMIT,
              begin?{probeOnly:true}:{evidenceCheckpoint:checkpoint});
          }catch {
            return {slot,status:'UNAVAILABLE',events:[]};
          }
        }));
        requireOpen();
        const windowSlots=slots.map((slot,index)=>{
          const report=reports[index];
          const present=report?.slot===slot&&report.status==='CAPTURE_FILE_PRESENT';
          if(begin&&present&&validCaptureCursor(report.captureCursor)) {
            evidenceBaselines.set(slot,Object.freeze({...report.captureCursor}));
            return {slot,status:'BASELINE_READY',
              baselinePartialLineExcluded:!report.captureCursor.completeLastLine};
          }
          if(!begin&&present&&Array.isArray(report.events)&&report.events.length<=EVIDENCE_LIMIT&&
             typeof report.windowCoverage?.recordLimitReached==='boolean'&&
             typeof report.windowCoverage?.baselinePartialLineExcluded==='boolean') {
            return {slot,status:'APPENDED_COMPLETE_LINES_ONLY',
              recordLimitReached:report.windowCoverage.recordLimitReached,
              baselinePartialLineExcluded:report.windowCoverage.baselinePartialLineExcluded,
              incompleteTrailingLineIgnored:report.incompleteTrailingLineIgnored===true};
          }
          const status=report?.slot===slot&&report.baselineRequired===true?'BASELINE_REQUIRED':
            report?.slot===slot&&['NOT_CONFIGURED','NO_CAPTURE_FILE','CAPTURE_FILE_UNSTABLE'].includes(report.status)?
              report.status:'UNAVAILABLE';
          // A definitive continuity failure permanently invalidates this
          // baseline. A transient live-write conflict may be read again.
          if(!begin&&report?.slot===slot&&(
             status==='NO_CAPTURE_FILE'||(status==='CAPTURE_FILE_UNSTABLE'&&
               ['EVIDENCE_CHECKPOINT_INVALID','EVIDENCE_CAPTURE_REPLACED',
                'EVIDENCE_CAPTURE_TRUNCATED','EVIDENCE_CAPTURE_TAIL_GAP',
                'EVIDENCE_CAPTURE_PREFIX_CHANGED','FILE_DISAPPEARED_DURING_READ'].includes(report.reason)))) {
            evidenceBaselines.delete(slot);
          }
          // Do not pass an invalid provider result to the summary projector.
          reports[index]={slot,status,events:[]};
          return {slot,status};
        });
        const captureWindow={mode:args.window,slots:windowSlots,
          scope:'LOCAL_CAPTURE_BYTES_NOT_VERIFIED_GAME_SESSION',
          timeline:'BOUNDED_SUMMARY_NOT_COMPLETE_GAMEPLAY_HISTORY'};
        if(begin)return {schema:1,captureWindow,
          verdict:'EVIDENCE_BASELINE_ONLY_NOT_GAMEPLAY_ACCEPTANCE'};
        return {...summarizeEvidenceReports(slots,reports),captureWindow};
      }finally {evidenceWindowBusy=false;}
  }
  async function read(args) {
      keysExactly(args,['slot']);
      const slot=slotOf(args.slot);
      // Preserve the public tool name and slot/status/events layout, but
      // never forward arbitrary debugger JSON fields to the model. Reuse
      // the existing allowlisted, bounded summary event projection.
      const summarized=summarizeEvidenceReports([slot],[
        await evidence(config.evidenceDirectory,slot),
      ]);
      const source=summarized.sources[0];
      return {
        schema:1,slot,status:source.status,
        events:summarized.timeline,records:summarized.records,
        truncated:source.truncated,
        incompleteTrailingLineIgnored:source.incompleteTrailingLineIgnored,
        attribution:'OPERATOR_NAMED_UNVERIFIED',
        response:'BOUNDED_PROJECTION_NO_RAW_SKILLS_PROBE_JSON',
        verdict:summarized.verdict,
        note:'Events contain bounded unverified metadata only, not raw captured payloads or gameplay acceptance.',
      };
  }
  return {summarize,read,clear:()=>evidenceBaselines.clear()};
}
