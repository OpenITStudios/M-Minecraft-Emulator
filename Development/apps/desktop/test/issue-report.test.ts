import {describe,expect,it} from 'vitest';
import {readFileSync} from 'node:fs';
import {IssueReport,MAX_ISSUES,reportableFeedback} from '../src/app/presentation/issueReport.js';
import {userFeedback,type UserMessageKey} from '../src/app/presentation/sessionFeedback.js';
import {parseNativeIssueReport,type ProfileSlot} from '../src/app/bridge/lazyTester.js';

describe('Selective desktop issue report',()=>{
  it('creates no report without an issue and deduplicates state without counting polls',()=>{
    let clockReads=0;
    const report=new IssueReport(()=>{clockReads++;return 100;});
    expect(report.read()).toBeNull();
    expect(clockReads).toBe(0);
    expect(report.note('LT-GAM-05',2)).toBe(true);
    for(let i=0;i<100;i++) expect(report.note('LT-GAM-05',2)).toBe(false);
    expect(clockReads).toBe(1);
    expect(report.note('LT-GAM-05',3)).toBe(true);
    expect(JSON.parse(report.read()!)).toEqual({v:1,scope:'desktop-observed',
      issues:[{c:'LT-GAM-05',s:2,t:100},{c:'LT-GAM-05',s:3,t:100}]});
  });
  it('counts discrete failed attempts and keeps first and last time',()=>{
    let time=100;
    const report=new IssueReport(()=>time);
    report.note('LT-CAP-01',1,true);
    time=120;
    report.note('LT-CAP-01',1,true);
    expect(JSON.parse(report.read()!).issues).toEqual([{c:'LT-CAP-01',s:1,t:100,n:2,l:120}]);
  });
  it('records recovery only for a retained issue and reopens on a genuine recurrence',()=>{
    let time=100;
    const report=new IssueReport(()=>time);
    report.resolve('LT-CAP-03');
    expect(report.read()).toBeNull();
    report.note('LT-CAP-03');
    time=110;
    report.resolve('LT-CAP-03');
    time=111;
    report.resolve('LT-CAP-03');
    expect(JSON.parse(report.read()!).issues).toEqual([{c:'LT-CAP-03',t:100,r:110}]);
    time=120;
    expect(report.note('LT-CAP-03')).toBe(true);
    expect(report.note('LT-CAP-03')).toBe(false);
    expect(JSON.parse(report.read()!).issues).toEqual([{c:'LT-CAP-03',t:100,e:1,l:120}]);
    report.note('LT-CAP-01',2,true);
    report.resolve('LT-CAP-01',1);
    report.resolve('LT-CAP-01',2);
    time=130;
    report.note('LT-CAP-01',2,true);
    expect(JSON.parse(report.read()!).issues[1]).toEqual({c:'LT-CAP-01',s:2,t:120,n:2,e:1,l:130});
  });
  it('checkpoints only meaningful changes and power-of-two counts',()=>{
    const report=new IssueReport(()=>100);
    expect(report.takeCheckpointDue()).toBe(false);
    for(const due of [true,true,false,true,false]) {
      report.note('LT-CAP-01',2,true);
      expect(report.takeCheckpointDue()).toBe(due);
    }
    report.resolve('LT-CAP-01',2);
    expect(report.takeCheckpointDue()).toBe(true);
    report.resolve('LT-CAP-01',2);
    expect(report.takeCheckpointDue()).toBe(false);
    report.note('LT-CAP-01',2,true);
    expect(report.takeCheckpointDue()).toBe(true);
  });
  it('retains recording failure until a positive same-slot saved receipt resolves it',()=>{
    let now=100;
    const report=new IssueReport(()=>now);
    report.note('LT-CAP-05',1);
    report.note('LT-CAP-05',2);
    now=110; report.resolve('LT-CAP-05',1);
    now=120; report.note('LT-CAP-05',1);
    expect(JSON.parse(report.read()!).issues).toEqual([
      {c:'LT-CAP-05',s:1,t:100,e:1,l:120},{c:'LT-CAP-05',s:2,t:100}]);
  });
  it('bounds the summary, preserves the first records and excludes expected refusals',()=>{
    const keys:UserMessageKey[]=['general','backend','binding','connection','launch','selection','workspace','location','storage','settings','performance'];
    const slots:ProfileSlot[]=[1,2,3,4,5];
    const report=new IssueReport(()=>100);
    for(const key of keys) for(const slot of slots) report.note(userFeedback(key).code,slot);
    const result=JSON.parse(report.read()!);
    expect(result.issues).toHaveLength(MAX_ISSUES);
    expect(result.issues[0]).toEqual({c:userFeedback('general').code,s:1,t:100});
    expect(result.truncated).toBe(true);
    expect(report.read()!.length).toBeLessThan(4096);
    for(const key of ['graphicsBlocked','performanceBlocked','closeRejected','closeAll'] as const)
      expect(reportableFeedback(key)).toBe(false);
  });
  it('combines bounded native evidence with UI observations without claiming correlation',()=>{
    const report=new IssueReport(()=>100);
    expect(report.readCombined(null)).toBeNull();
    const native=parseNativeIssueReport({v:1,issues:[{c:'IPC_REPLY_TIMEOUT',op:'close_one',s:2,t:100,n:2,l:120}]});
    expect(JSON.parse(report.readCombined(native)!)).toEqual({v:2,scope:'desktop-process',issues:[],native});
    report.note('LT-CON-01');
    expect(JSON.parse(report.readCombined(native)!)).toEqual({v:2,scope:'desktop-process',
      issues:[{c:'LT-CON-01',t:100}],native});
    expect(JSON.parse(report.readCombined(undefined)!)).toMatchObject({nativeUnavailable:true});
    expect(JSON.parse(report.readCombined(null)!)).toMatchObject({native:null});
    for(const change of [{op:'unknown'},{c:'unknown'},{s:9},{raw:'secret'},{n:0},{l:null}])
      expect(()=>parseNativeIssueReport({v:1,issues:[{...native!.issues[0],...change}]})).toThrow();
    expect(()=>parseNativeIssueReport({v:1,issues:Array(17).fill(native!.issues[0])})).toThrow();
    const recovered={...native!.issues[0],r:130,e:1};
    expect(parseNativeIssueReport({v:1,issues:[recovered]})?.issues[0]).toEqual(recovered);
    for(const change of [{r:'130'},{e:0},{e:2},{c:'IPC_WORKER_START_FAILED'}])
      expect(()=>parseNativeIssueReport({v:1,issues:[{...recovered,...change}]})).toThrow();
  });
  it('allows explicit native-only review and uses existing feedback without exporting raw data',()=>{
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const settings=readFileSync(new URL('../src/GeneralSettings.svelte',import.meta.url),'utf8');
    expect(app).toContain('if (issueKey!==null && reportableFeedback(issueKey))');
    expect(app).toContain('if (closeFailure) noteIssue(userFeedback(\'closeAll\').code)');
    expect(app).toContain('{issueReportAvailable} onReadIssueReport={readIssueReport}');
    expect(settings).toContain("issueReportAvailable?'Copy report':'Check issue report'");
    expect(settings).toContain("issueReportNotice='Issue report unavailable. Try again.'");
    expect(app).toContain("if (partial===null) throw new Error('Issue report unavailable')");
    expect(app).not.toContain('if (ui===null) continue;');
    expect(settings).toContain('await navigator.clipboard.writeText(issueReportText)');
    expect(app).toContain('await lazyTester.issueReport()');
    expect(app).toContain('await lazyTester.checkpointIssues(ui)');
    expect(app).toContain('if (issueCheckpoint !== null) return;');
    expect(app).toContain("resolveIssue('LT-CAP-03')");
    const recording=app.slice(app.indexOf("async function refreshRecordingStatus()"),app.indexOf("async function recordingAction()"));
    expect(recording).toContain("if (item.state==='Failed') noteIssue('LT-CAP-05',item.slot)");
    expect(recording).toContain("else if (item.state==='Saved') resolveIssue('LT-CAP-05',item.slot)");
    const shot=app.slice(app.indexOf('async function saveScreenshot()'),app.indexOf('let manualRefreshRequested'));
    expect(shot.indexOf('if(receipt&&!disposed)')).toBeLessThan(shot.indexOf("resolveIssue('LT-CAP-01',slot)"));
    const session=readFileSync(new URL('../src-tauri/src/engine/native_clients_session.rs',import.meta.url),'utf8');
    const locked=session.slice(session.indexOf('fn request_locked('),session.indexOf('// Only the desktop\'s passive idle lane'));
    expect(locked).toContain('if reply.is_ok() {\n                            if owner.uncertain {');
    expect(locked.indexOf('!snapshot.is_full_inspect_receipt()')).toBeLessThan(locked.indexOf('issue_report::reconcile_transport();'));
    expect(locked.match(/issue_report::reconcile_transport\(\)/g)).toHaveLength(1);
    expect(settings).toContain('await onReadIssueReport()');
    expect(settings).toContain('readonly rows="4" value={issueReportText}');
  });
});
