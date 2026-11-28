import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {issueCatalogCodes,readIssueReport,reviewIssueReport} from './report-review.mjs';

const sample=()=>({v:2,scope:'desktop-process',issues:[{c:'LT-CON-01',t:100}],
  native:{v:1,issues:[{c:'IPC_REPLY_TIMEOUT',op:'close_one',s:2,t:100,n:2,l:105,r:110}]}});
const review=value=>reviewIssueReport(JSON.stringify(value));

test('issue review keeps native and UI observations separate, with no inferred root cause or build',()=>{
  const result=review(sample());
  assert.equal(result.status,'REVIEW_CANDIDATES');
  assert.equal(result.reportedBuild,'UNKNOWN');
  assert.equal(result.rootCause,'NOT_ESTABLISHED');
  assert.equal(result.coverage.uiRows,1);
  assert.equal(result.coverage.nativeRows,1);
  assert.equal(result.issues[0].producer,'native-request-client');
  assert.equal(result.issues[0].recovery,'TRANSPORT_RECONCILED');
  assert.equal(result.issues[1].recovery,'UNCONFIRMED');
  assert.ok(result.routes[0].source.endsWith('native_clients_session.rs'));
});
test('review bounds output, ranks unresolved recurrences, preserves unknown coverage',()=>{
  const value={v:2,scope:'desktop-process',issues:[],nativeUnavailable:true,truncated:true};
  for(const c of ['LT-DIS-03','LT-GAM-02','LT-STO-06'])for(let s=1;s<=5;s++)
    value.issues.push({c,s,t:100});
  value.issues[14].e=3;
  value.issues[14].l=120;
  const result=review(value);
  assert.equal(result.coverage.nativeRows,null);
  assert.equal(result.coverage.truncated,true);
  assert.equal(result.coverage.omitted,3);
  assert.equal(result.issues.length,12);
  assert.equal(result.issues[0].e,3);
});
test('rejects raw, unknown, malformed, oversized and duplicate report data',()=>{
  for(const change of [{c:'PRIVATE'}, {op:'close_one'}, {raw:'secret'}, {s:6}, {r:null}, {n:0}]){
    const v=sample();Object.assign(v.issues[0],change);assert.throws(()=>review(v),/Invalid/);
  }
  for(const change of [{c:'IPC_FAKE'}, {op:'private'}, {n:0}, {e:2}, {s:6}, {os:2147483648}]){
    const v=sample();Object.assign(v.native.issues[0],change);assert.throws(()=>review(v),/Invalid/);
  }
  const v=sample();v.issues.push({...v.issues[0]});assert.throws(()=>review(v),/Invalid/);
  assert.throws(()=>reviewIssueReport(' '.repeat(16385)),/Invalid/);
  assert.throws(()=>review({v:2,scope:'desktop-process',issues:[],native:null}),/Invalid/);
});
test('fixed checkpoint reader never consumes pending staging or oversized files',()=>{
  const root=mkdtempSync(join(tmpdir(),'lt-report-review-'));
  try{
    assert.equal(readIssueReport(root),null);
    const dir=join(root,'Data','Reports');mkdirSync(dir,{recursive:true});
    writeFileSync(join(dir,'issues.pending'),'private');
    assert.equal(readIssueReport(root),null);
    const file=join(dir,'issues.json');
    writeFileSync(file,JSON.stringify(sample()));
    assert.equal(reviewIssueReport(readIssueReport(root)).status,'REVIEW_CANDIDATES');
    writeFileSync(file,'x'.repeat(16385));
    assert.throws(()=>readIssueReport(root),/Invalid/);
  }finally{rmSync(root,{recursive:true,force:true});}
});
test('review catalog covers current storage/UI/native code vocabulary and source paths',()=>{
  const store=readFileSync(new URL('../../apps/desktop/src-tauri/src/engine/report_store.rs',import.meta.url),'utf8');
  const bridge=readFileSync(new URL('../../apps/desktop/src/app/bridge/lazyTester.ts',import.meta.url),'utf8');
  const uiCodes=[...store.matchAll(/"(LT-[A-Z]+-[0-9]+)"/g)].map(m=>m[1]);
  const nativeCodes=JSON.parse(bridge.split('\n').find(line=>line.startsWith('const nativeIssueCodes = ')).split(' = ')[1].split(' as const')[0]);
  const nativeOps=JSON.parse(bridge.split('\n').find(line=>line.startsWith('const nativeIssueOperations = ')).split(' = ')[1].split(' as const')[0]);
  const native=readFileSync(new URL('../../apps/desktop/src-tauri/src/engine/issue_report.rs',import.meta.url),'utf8');
  const rustNames=name=>native.split('enum '+name+' {')[1].split('}')[0].trim().split(',').map(v=>v.trim()).filter(Boolean);
  const snake=s=>s.replace(/[A-Z]/g,(c,i)=>(i?'_':'')+c.toLowerCase());
  assert.deepEqual(rustNames('Operation').map(snake).sort(),nativeOps.sort());
  assert.deepEqual(rustNames('Code').map(s=>snake(s).toUpperCase()).sort(),[...nativeCodes].sort());
  assert.deepEqual([...issueCatalogCodes()].sort(),[...new Set([...uiCodes,...nativeCodes])].sort());
  for(const code of issueCatalogCodes()){
    const value=code.startsWith('IPC_')?{v:2,scope:'desktop-process',issues:[],native:{v:1,issues:[{c:code,op:'inspect',t:1,n:1}]}}:
      {v:2,scope:'desktop-process',issues:[{c:code,t:1}],native:null};
    for(const route of review(value).routes)assert.ok(readFileSync(new URL('../../../'+route.source,import.meta.url),'utf8').length);
  }
});
