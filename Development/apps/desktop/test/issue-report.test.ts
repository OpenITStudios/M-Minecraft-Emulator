import {describe,expect,it} from 'vitest';
import {readFileSync} from 'node:fs';
import {IssueReport,MAX_ISSUES,reportableFeedback} from '../src/app/presentation/issueReport.js';
import {userFeedback,type UserMessageKey} from '../src/app/presentation/sessionFeedback.js';
import type {ProfileSlot} from '../src/app/bridge/lazyTester.js';

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
  it('mounts explicit copy only after a problem and uses existing feedback without exporting raw data',()=>{
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const settings=readFileSync(new URL('../src/GeneralSettings.svelte',import.meta.url),'utf8');
    expect(app).toContain('if (issueKey!==null && reportableFeedback(issueKey))');
    expect(app).toContain('if (closeFailure) noteIssue(userFeedback(\'closeAll\').code)');
    expect(app).toContain('{issueReportAvailable} onReadIssueReport={readIssueReport}');
    expect(settings).toContain('{#if issueReportAvailable}');
    expect(settings).toContain('await navigator.clipboard.writeText(issueReportText)');
    expect(settings).toContain('readonly rows="4" value={issueReportText}');
  });
});
