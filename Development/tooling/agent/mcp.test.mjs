import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,mkdir,readFile,writeFile,symlink,rm,open,rename,appendFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {parseConfiguration,parseCapturedLine,readGameEvidence,createHandler,coordinateTestingRunNative,inspectTestingRunNative,parseTestingInspectionReceipt,testingRecoveryNextAction,validateNativeReport,validateNativeWindowTarget,boundedMcpText,sendExperimentalMc02Native,fingerprintActions,
  sendExperimentalAiWriterLeaseNative,sameNativeVisualTarget,makeAiLeaseEpoch} from './mcp.mjs';

const snapshot=(operation)=>({schema:1,ok:true,operation,profiles:Array.from({length:5},(_,i)=>({slot:i+1,state:i===0?'RUNNING':'STOPPED',responding:i===0,hasWindow:i===0,needsAttention:false}))});
const config=(mode='human-plus-four-ai')=>({mode,executable:'C:\\App\\lazy-tester.exe',evidenceDirectory:''});
const leaseAck=async(_exe,operation,slots,
    {writerId,mode,leaseEpoch,neutralOnly}={})=>{
  assert.match(writerId,/^[a-f0-9]{32}$/);
  assert.match(leaseEpoch,/^[a-f0-9]{30}e1$/);
  assert.equal(['human-plus-four-ai','five-ai'].includes(mode),true);
  assert.equal(typeof neutralOnly,'boolean');
  return {schema:1,slots,status:operation==='acquire'?
    'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
    gameplayObserved:false};
};

test('modes, absolute paths, no implicit native permissions',()=>{
  assert.equal(parseConfiguration(['--mode','five-ai']).mode,'five-ai');
  assert.equal(parseConfiguration([]).mode,'human-plus-four-ai');
  assert.throws(()=>parseConfiguration(['--mode','random']));
  assert.throws(()=>parseConfiguration(['--evidence-dir','relative']));
  assert.throws(()=>parseConfiguration(['--exe','./lazy-tester.exe']));
  assert.throws(()=>parseConfiguration(['--cua-exe','./cua-driver.exe']));
  assert.throws(()=>parseConfiguration(['--gpu-capture-exe','gpu.exe']));
  assert.throws(()=>parseConfiguration(['--probe-zip','relative.zip']));
  assert.equal(parseConfiguration(['--probe-zip',path.resolve('skills-probe.zip')])
    .probeZip,path.resolve('skills-probe.zip'));
  assert.equal(parseConfiguration([
    '--gpu-capture-exe',path.resolve('lazy-tester-gpu-capture.exe')])
    .gpuExecutable,path.resolve('lazy-tester-gpu-capture.exe'));
  assert.equal(parseConfiguration(['--cua-exe',path.resolve('cua-driver.exe')])
    .cuaExecutable,path.resolve('cua-driver.exe'));
});

test('MCP handshake and tool discovery are standard JSON-RPC',async()=>{
  const h=createHandler(config());
  const init=await h.handle({jsonrpc:'2.0',id:1,method:'initialize',params:{protocolVersion:'2025-11-25'}});
  assert.equal(init.result.protocolVersion,'2025-11-25');
  const listing=await h.handle({jsonrpc:'2.0',id:2,method:'tools/list'});
  assert.equal(listing.result.tools.find(x=>x.name==='probe_qa')
    .inputSchema.properties.publishToDesktop.type,'boolean');
  assert.match(listing.result.tools.find(x=>x.name==='prepare_probe_qa')
    .description,/LEGACY COMPATIBILITY ONLY/);
  assert.match(listing.result.tools.find(x=>x.name==='read_probe_qa_result')
    .description,/LEGACY COMPATIBILITY ONLY/);
  assert.deepEqual(listing.result.tools.map(x=>x.name),['start_visual_observation','get_multi_client_visual_state','read_visual_observation','stop_visual_observation','observe_client','get_computer_use_target','revalidate_computer_use_target','get_test_context','agent_capabilities','inspect_profiles','launch_profile','plan_peer_session','plan_player_actions','assess_input_route','summarize_test_evidence','read_game_evidence','probe_qa','prepare_probe_qa','read_probe_qa_result','get_diagnostics_overview','list_issue_candidates','get_issue_details','query_diagnostic_evidence']);
  assert.equal(await h.handle({jsonrpc:'2.0',method:'notifications/initialized'}),null);
});

test('diagnostics MCP tools are read-only and unconfigured by default',async()=>{
  const handler=createHandler(config(),{
    native:async()=>{throw new Error('native must not be contacted');},
  });
  const overview=await handler.handle({jsonrpc:'2.0',id:99,method:'tools/call',
    params:{name:'get_diagnostics_overview',arguments:{}}});
  assert.equal(overview.result.isError,false);
  const result=JSON.parse(overview.result.content[0].text);
  assert.equal(result.status,'NOT_CONFIGURED');
  assert.equal(result.rawDataSharedWithAI,false);
  const candidate=await handler.handle({jsonrpc:'2.0',id:100,method:'tools/call',
    params:{name:'list_issue_candidates',arguments:{run:'qa-run'}}});
  assert.equal(JSON.parse(candidate.result.content[0].text).status,'NOT_CONFIGURED');
  assert.throws(()=>parseConfiguration(['--diagnostics-dir','relative'],{}));
  assert.throws(()=>parseConfiguration(['--diagnostics-dir',path.resolve('elsewhere')],{}));
});

test('QA MCP requires one pinned ZIP and cannot target human MC01',async()=>{
  let nativeCalls=0;
  const h=createHandler(config(),{native:async()=>{
    nativeCalls++;throw Error('must not call native for QA source prep');
  }});
  const initial=await h.callTool('prepare_probe_qa',{});
  assert.equal(initial.status,'PROBE_ZIP_NOT_CONFIGURED');
  assert.equal(initial.commandSent,false);
  await assert.rejects(()=>h.callTool('prepare_probe_qa',{slot:1}),
    /MC01 human host is protected/);
  const results=await h.callTool('read_probe_qa_result',{
    slot:2,sessionId:'a'.repeat(24)});
  assert.equal(results.status,'NOT_PREPARED_OR_EXPIRED');
  assert.equal(nativeCalls,0);
});

test('one AI QA MCP entry performs source/capture/target readiness then compact read',async()=>{
  let targetBirth='1234567';
  const nativeCalls=[];
  const native=async(_exe,operation,slot)=>{
    nativeCalls.push([operation,slot]);
    assert.equal(operation,'target');
    assert.equal(slot,2);
    return {schema:1,ok:true,operation:'target',slot,
      pid:4002,windowId:8002,createdFileTime:targetBirth,
      sessionId:1,foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED'};
  };
  const checkpoint={kind:'STABLE_CAPTURE_CURSOR_V1'};
  let catalogReads=0,revisionReads=0;
  const catalog=async()=>{
    catalogReads++;
    return {autoCount:2,autoIds:['P01','P02'],
      archiveSha256:'a'.repeat(64),runnerSha256:'e'.repeat(64),
      sourceRevision:'f'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'};
  };
  const evidence=async(_directory,slot,_limit,options)=>{
    if(options?.probeCheckpoint)return {slot,status:'CAPTURE_FILE_PRESENT',
      probeEvents:[
        {kind:'start',arg:'auto',count:2,tick:100,at:null},
        {kind:'result',id:'P01',result:'pass',tick:101,at:null},
        {kind:'result',id:'P02',result:'pass',tick:102,at:null},
        {kind:'done',arg:'auto',count:2,tick:103,at:null},
      ]};
    return {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:checkpoint};
  };
  const h=createHandler({...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{native,evidence,probeCatalog:catalog,
      probeRevisionStatus:async()=>{revisionReads++;return 'UNCHANGED';}});
  const ready=await h.callTool('probe_qa',{slot:2});
  assert.equal(ready.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(ready.status,'BLOCKED');
  assert.equal(ready.probeCount,2);
  assert.equal(ready.command,'/scriptevent probe:run auto');
  assert.equal(ready.target,'NATIVE_TARGET_OBSERVED_NOT_GAME_READY');
  assert.deepEqual(ready.targetHandoff.window,{kind:'window',pid:4002,window_id:8002});
  assert.equal(ready.targetHandoff.identity.createdFileTime,'1234567');
  assert.equal(ready.targetHandoff.verification,'PROCESS_WINDOW_ONLY_NOT_GAME_WORLD');
  assert.equal(ready.targetHandoff.input,'NOT_AVAILABLE_OR_AUTHORIZED');
  assert.equal(ready.commandSentByLazyTester,false);
  assert.equal(ready.reason,'COMMAND_DISPATCH_NOT_ENABLED');
  assert.equal(ready.sourceCapture,'APPROVED_SOURCE_LOG_FILE_PRESENT');
  assert.equal(ready.gameExecution,'NOT_VERIFIED');
  const qaPending=(await h.callTool('agent_capabilities',{})).activity;
  assert.deepEqual(qaPending.items.map(x=>[x.kind,x.slot,x.status]),[
    ['SKILLS_PROBE',2,'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE']]);
  assert.equal(qaPending.desktopControl,'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
  assert.equal(qaPending.mapGameplayAcceptance,'NOT_TESTED');
  assert.equal(ready.mapPlan,undefined);
  assert.equal(ready.qaReport,undefined); // legacy source-only shape stays unextended
  assert.equal(ready.proofGate.readiness,'BLOCKED');
  assert.equal(ready.proofGate.targetAtPrepare,'NATIVE_PROCESS_WINDOW_ONLY');
  assert.equal(ready.proofGate.playerAdmin,'NOT_VERIFIED');
  assert.equal(ready.proofGate.qaWorld,'NOT_VERIFIED');
  assert.equal(ready.proofGate.automaticDispatch,'DISABLED');
  assert.deepEqual(nativeCalls,[['target',2]]); // read-only; no launch/input
  const report=await h.callTool('probe_qa',{slot:2});
  assert.equal(report.status,'PROBE_REPORTED_PASS');
  assert.equal(report.terminal,true);
  const qaActivity=(await h.callTool('agent_capabilities',{})).activity;
  assert.equal(qaActivity.items[0].status,'PROBE_REPORT_AVAILABLE');
  assert.equal(qaActivity.items[0].reportedOutcome,'PROBE_REPORTED_PASS');
  assert.deepEqual(qaActivity.items[0].probeCounts,
    {expected:2,pass:2,fail:0,info:0,look:0});
  assert.equal(qaActivity.mapGameplayAcceptance,'NOT_TESTED');
  assert.equal(report.qaReport,undefined);
  assert.equal(report.probeReportedStatus,undefined);
  assert.equal(report.counts.pass,2);
  assert.equal(report.target,'NATIVE_PROCESS_MATCH_AT_READ_TIME_ONLY');
  assert.equal(report.gameplayAcceptance,'NOT_VERIFIED');
  assert.deepEqual(report.proofGate,ready.proofGate);
  assert.equal(report.proofGate.logOrigin,'NOT_VERIFIED');
  assert.deepEqual(nativeCalls,[['target',2],['target',2]]);
  assert.equal(catalogReads,1); // No catalog reload on result reads
  assert.equal(revisionReads,1); // Only cheap metadata comparison
  // A completed report is a historical snapshot; it does not imply
  // the currently running Minecraft process remains the same.
  targetBirth='9876543';
  const cached=await h.callTool('probe_qa',{slot:2});
  assert.deepEqual(cached,report);
  assert.deepEqual(nativeCalls,[['target',2],['target',2]]);
  assert.equal(catalogReads,1);
  assert.equal(revisionReads,1);
  const restarted=await h.callTool('probe_qa',{slot:2,restart:true});
  assert.equal(restarted.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(restarted.commandSentByLazyTester,false);
  assert.equal(restarted.targetHandoff.identity.createdFileTime,'9876543');
  assert.equal(catalogReads,2); // only the intentional new session prepares
  await assert.rejects(()=>h.callTool('probe_qa',{slot:2,restart:'true'}),
    /Invalid tool arguments/);
  h.shutdown();
});

test('unchanged Probe status polls skip ZIP parsing and native target IPC',async()=>{
  let catalogReads=0,revisionReads=0,nativeReads=0,captureReads=0;
  const h=createHandler({...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>{
      catalogReads++;
      return {autoIds:['P01'],autoCount:1,archiveSha256:'a'.repeat(64),
        runnerSha256:'b'.repeat(64),sourceRevision:'c'.repeat(64),
        cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=>{revisionReads++;return 'UNCHANGED';},
    native:async(_exe,op,slot)=>{
      nativeReads++;
      return {schema:1,ok:true,operation:'target',slot,pid:402,
        windowId:802,createdFileTime:'9001',sessionId:1,
        foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
    evidence:async(_dir,slot,_limit,opts)=>{
      captureReads++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[]}:
        {slot,status:'CAPTURE_FILE_PRESENT',
          captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  assert.equal(catalogReads,1);
  assert.equal(nativeReads,1);
  const first=await h.callTool('probe_qa',{slot:2});
  const second=await h.callTool('probe_qa',{slot:2});
  assert.equal(first.status,'WAITING');
  assert.equal(second.status,'WAITING');
  assert.equal(catalogReads,1); // no ZIP open/hash on status polls
  assert.equal(revisionReads,2); // cheap metadata-change probes only
  assert.equal(nativeReads,1); // no repeated target IPC while waiting
  assert.equal(captureReads,3);
  h.shutdown();
});

test('terminal Probe report is reused after 20 minutes without repeated IO',async()=>{
  let qaTime=Date.parse('2026-11-02T07:00:00.000Z');
  let catalogReads=0,revisionReads=0,evidenceReads=0,nativeReads=0;
  const h=createHandler({...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    qaClock:()=>qaTime,
    probeCatalog:async()=>{
      catalogReads++;
      return {autoCount:1,autoIds:['P01'],archiveSha256:'a'.repeat(64),
        runnerSha256:'b'.repeat(64),sourceRevision:'c'.repeat(64),
        cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=>{revisionReads++;return 'UNCHANGED';},
    evidence:async(_dir,slot,_n,options)=>{
      evidenceReads++;
      return options?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:1,tick:1,at:null},
          {kind:'result',id:'P01',result:'fail',tick:2,at:null},
          {kind:'done',arg:'auto',count:1,tick:3,at:null},
        ]}:
        {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
          kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
    native:async(_exe,op,slot)=>{
      nativeReads++;
      return {schema:1,ok:true,operation:'target',slot,pid:202,
        windowId:808,createdFileTime:'5555',sessionId:1,
        foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
  });
  const prepared=await h.callTool('probe_qa',{slot:2});
  assert.equal(prepared.sourceCapture,'APPROVED_SOURCE_LOG_FILE_PRESENT');
  assert.equal(prepared.gameExecution,'NOT_VERIFIED');
  const finished=await h.callTool('probe_qa',{slot:2});
  assert.equal(finished.status,'FAIL');
  assert.equal(finished.terminal,true);
  assert.equal(finished.counts.fail,1);
  const before=[catalogReads,revisionReads,evidenceReads,nativeReads];
  qaTime+=25*60*1000;
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),finished);
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),finished);
  assert.deepEqual([catalogReads,revisionReads,evidenceReads,nativeReads],before);
  const fresh=await h.callTool('probe_qa',{slot:2,restart:true});
  assert.equal(fresh.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(fresh.terminal,undefined);
  assert.equal(catalogReads,before[0]+1);
  h.shutdown();
});

test('expired pending run is held, never implicitly prepared again',async()=>{
  let now=Date.parse('2026-11-02T07:00:00.000Z');
  let catalogs=0,captures=0,revisions=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    qaClock:()=>now,
    probeCatalog:async()=>{
      catalogs++;
      return {autoCount:1,autoIds:['P01'],archiveSha256:'a'.repeat(64),
        runnerSha256:'b'.repeat(64),sourceRevision:'c'.repeat(64),
        cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=>{revisions++;return 'UNCHANGED';},
    evidence:async(_dir,slot,_n,opts)=>{
      captures++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[]}:
        {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
          kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  now+=20*60*1000+1;
  const expired=await h.callTool('probe_qa',{slot:2});
  assert.equal(expired.status,'BLOCKED');
  assert.equal(expired.reason,'QA_PLAN_EXPIRED');
  assert.equal(expired.terminal,true);
  const counters=[catalogs,captures,revisions];
  now+=2*60*60*1000;
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),expired);
  assert.deepEqual([catalogs,captures,revisions],counters);
  const refreshed=await h.callTool('probe_qa',{slot:2,restart:true});
  assert.equal(refreshed.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(catalogs,counters[0]+1);
  h.shutdown();
});

test('source change during pending QA stops once and preserves stop report',async()=>{
  let status='UNCHANGED',catalogReads=0,captureReads=0,revisionReads=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>{
      catalogReads++;
      return {autoCount:1,autoIds:['P01'],archiveSha256:'a'.repeat(64),
        runnerSha256:'b'.repeat(64),sourceRevision:'c'.repeat(64),
        cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=>{revisionReads++;return status;},
    evidence:async(_dir,slot)=>{
      captureReads++;
      return {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
        kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  status='CHANGED';
  const stopped=await h.callTool('probe_qa',{slot:2});
  assert.equal(stopped.status,'BLOCKED');
  assert.equal(stopped.reason,'QA_SOURCE_CHANGED_STOP');
  assert.equal(stopped.terminal,true);
  status='UNCHANGED';
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),stopped);
  assert.equal(catalogReads,1);
  assert.equal(captureReads,1);
  assert.equal(revisionReads,1);
  assert.equal((await h.callTool('probe_qa',{slot:2,restart:true}))
    .phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(catalogReads,2);
  h.shutdown();
});

test('irrecoverable runner gaps settle once and require explicit new run',async()=>{
  let reads=0,sourceChecks=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({autoCount:2,autoIds:['P01','P02'],
      archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'}),
    probeRevisionStatus:async()=>{sourceChecks++;return 'UNCHANGED';},
    evidence:async(_dir,slot,_n,opts)=>{
      reads++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:2,tick:100,at:null},
          {kind:'result',id:'P01',result:'pass',tick:101,at:null},
          {kind:'done',arg:'auto',count:2,tick:102,at:null},
        ]}:
        {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
          kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  const incomplete=await h.callTool('probe_qa',{slot:2});
  assert.equal(incomplete.status,'INCOMPLETE');
  assert.equal(incomplete.reason,'INCOMPLETE_RESULTS');
  assert.equal(incomplete.terminal,true);
  assert.deepEqual(incomplete.missingIds,['P02']);
  const before=[reads,sourceChecks];
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),incomplete);
  assert.deepEqual([reads,sourceChecks],before);
  await h.callTool('probe_qa',{slot:2,restart:true});
  assert.equal(reads,before[0]+1);
  h.shutdown();
});

test('target replacement before first result remains incomplete and terminal',async()=>{
  let birth='111111',nativeReads=0,evidenceReads=0;
  const h=createHandler({...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'}),
    probeRevisionStatus:async()=> 'UNCHANGED',
    native:async(_exe,op,slot)=>{
      nativeReads++;
      return {schema:1,ok:true,operation:'target',slot,pid:302,
        windowId:902,createdFileTime:birth,sessionId:1,
        foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
    evidence:async(_dir,slot,_n,opts)=>{
      evidenceReads++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:1,tick:100,at:null},
          {kind:'result',id:'P01',result:'pass',tick:101,at:null},
          {kind:'done',arg:'auto',count:1,tick:102,at:null},
        ]}:
        {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
          kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  birth='222222';
  const result=await h.callTool('probe_qa',{slot:2});
  assert.equal(result.status,'INCOMPLETE');
  assert.equal(result.reason,'QA_NATIVE_TARGET_CHANGED');
  assert.equal(result.terminal,true);
  const before=[nativeReads,evidenceReads];
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),result);
  assert.deepEqual([nativeReads,evidenceReads],before);
  h.shutdown();
});

test('capture replacement stops once; a transient capture read may retry',async()=>{
  let captureStatus='FILE_CHANGED_DURING_READ',reads=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'}),
    probeRevisionStatus:async()=> 'UNCHANGED',
    evidence:async(_dir,slot,_n,opts)=>{
      reads++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_UNSTABLE',reason:captureStatus,
          probeEvents:[]}:
        {slot,status:'CAPTURE_FILE_PRESENT',
          captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  await h.callTool('probe_qa',{slot:2});
  const transient=await h.callTool('probe_qa',{slot:2});
  assert.equal(transient.status,'INCOMPLETE');
  assert.equal(transient.terminal,undefined);
  const initialReads=reads;
  captureStatus='PROBE_CAPTURE_REPLACED';
  const stopped=await h.callTool('probe_qa',{slot:2});
  assert.equal(stopped.status,'INCOMPLETE');
  assert.equal(stopped.captureIssue,'PROBE_CAPTURE_REPLACED');
  assert.equal(stopped.terminal,true);
  assert.equal(reads,initialReads+1);
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),stopped);
  assert.equal(reads,initialReads+1);
  h.shutdown();
});

test('one QA entry blocks missing setup and protects human MC01',async()=>{
  const h=createHandler({...config(),executable:''},{
    native:async()=>{throw Error('native must not be invoked');},
  });
  const missing=await h.callTool('probe_qa',{});
  assert.equal(missing.status,'BLOCKED');
  assert.equal(missing.reason,'PROBE_ZIP_NOT_CONFIGURED');
  assert.equal(missing.gameplayAcceptance,'NOT_VERIFIED');
  await assert.rejects(()=>h.callTool('probe_qa',{slot:1}),
    /MC01 human host is protected/);
  h.shutdown();
});

test('one QA entry refuses concurrent source preparations for the same slot',async()=>{
  let release;
  const pendingCatalog=new Promise(resolve=>{release=resolve;});
  const catalog={autoCount:1,autoIds:['P01'],
    archiveSha256:'b'.repeat(64),runnerSha256:'e'.repeat(64),
    sourceRevision:'f'.repeat(64),cache:'FRESH_SOURCE_REVIEW'};
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>{await pendingCatalog;return catalog;},
    probeRevisionStatus:async()=> 'UNCHANGED',
    evidence:async(_directory,slot)=>({slot,status:'CAPTURE_FILE_PRESENT',
      captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}}),
  });
  const first=h.callTool('probe_qa',{slot:2});
  const denied=await h.callTool('probe_qa',{slot:2});
  assert.equal(denied.status,'BLOCKED');
  assert.equal(denied.reason,'QA_OPERATION_IN_PROGRESS');
  release();
  const ready=await first;
  assert.equal(ready.phase,'AWAITING_AUTHORIZED_COMMAND');
  h.shutdown();
});

test('QA single entry refuses source drift and log-only PASS without a pinned target',async()=>{
  let digest='a'.repeat(64),failRead=false;
  const expected={autoCount:2,autoIds:['P01','P02'],
    runnerSha256:'e'.repeat(64),sourceRevision:'f'.repeat(64),
    cache:'UNCHANGED_APPROVED_ZIP'};
  const catalog=async()=>{if(failRead)throw Error('missing ZIP');return {
    ...expected,archiveSha256:digest};};
  const cp={kind:'STABLE_CAPTURE_CURSOR_V1'};
  const events=[
    {kind:'start',arg:'auto',count:2,tick:100,at:null},
    {kind:'result',id:'P01',result:'pass',tick:101,at:null},
    {kind:'result',id:'P02',result:'pass',tick:102,at:null},
    {kind:'done',arg:'auto',count:2,tick:103,at:null},
  ];
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:catalog,
    probeRevisionStatus:async()=>failRead?'UNAVAILABLE':
      digest==='a'.repeat(64)?'UNCHANGED':'CHANGED',
    native:async()=>{throw Error('Unexpected native target inspection');},
    evidence:async(_dir,slot,_limit,opts)=>opts?.probeCheckpoint?
      {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:events}:
      {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:cp},
  });
  assert.equal((await h.callTool('probe_qa',{slot:2})).phase,'AWAITING_AUTHORIZED_COMMAND');
  const logOnly=await h.callTool('probe_qa',{slot:2});
  assert.equal(logOnly.status,'INCOMPLETE');
  assert.equal(logOnly.reason,'QA_NATIVE_TARGET_NOT_BOUND');
  assert.equal(logOnly.counts.pass,2);
  assert.equal(logOnly.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(logOnly.terminal,true);
  digest='b'.repeat(64);
  // No automatic rerun or reinterpretation of a finalized snapshot.
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),logOnly);
  const renewed=await h.callTool('probe_qa',{slot:2,restart:true});
  assert.equal(renewed.phase,'AWAITING_AUTHORIZED_COMMAND');
  failRead=true;
  const unavailable=await h.callTool('probe_qa',{slot:2});
  assert.equal(unavailable.status,'BLOCKED');
  assert.equal(unavailable.reason,'QA_SOURCE_UNAVAILABLE');
  assert.equal(unavailable.terminal,true);
  assert.deepEqual(await h.callTool('probe_qa',{slot:2}),unavailable);
  h.shutdown();
});

test('one QA entry fails closed on unreviewed ZIP and missing result source',async()=>{
  const base={...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')};
  const blocked=createHandler(base,{
    probeCatalog:async()=>{throw Error('PROBE_SOURCE_REVIEW_REQUIRED');},
    evidence:async()=>{throw Error('not expected after rejected ZIP');},
  });
  const refused=await blocked.callTool('probe_qa',{slot:2});
  assert.equal(refused.status,'BLOCKED');
  assert.equal(refused.reason,'PROBE_SOURCE_REVIEW_REQUIRED');
  blocked.shutdown();

  let stage=0;
  const ready=createHandler(base,{
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      archiveSha256:'c'.repeat(64),runnerSha256:'e'.repeat(64),
      sourceRevision:'f'.repeat(64),cache:'FRESH_SOURCE_REVIEW'}),
    probeRevisionStatus:async()=> 'UNCHANGED',
    evidence:async(_dir,slot,_limit,options)=>{
      if(options?.probeCheckpoint)throw Error('capture disappeared');
      stage++;
      return {slot,status:'CAPTURE_FILE_PRESENT',
        captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  });
  const prepared=await ready.callTool('probe_qa',{slot:2});
  assert.equal(prepared.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(stage,1);
  const incomplete=await ready.callTool('probe_qa',{slot:2});
  assert.equal(incomplete.status,'INCOMPLETE');
  assert.equal(incomplete.reason,'EVIDENCE_UNAVAILABLE');
  assert.equal(incomplete.gameplayAcceptance,'NOT_VERIFIED');
  ready.shutdown();
});

test('QA source catalog refuses a missing runner checksum before capture',async()=>{
  let reads=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({
      autoCount:1,autoIds:['P01'],archiveSha256:'a'.repeat(64),
      cache:'FRESH_SOURCE_REVIEW'}),
    evidence:async()=>{reads++;throw Error('capture must not be read');},
  });
  const blocked=await h.callTool('probe_qa',{slot:2});
  assert.equal(blocked.status,'BLOCKED');
  assert.equal(blocked.reason,'PROBE_SOURCE_REVIEW_REQUIRED');
  assert.equal(reads,0);
  h.shutdown();
});

test('hybrid mode refuses to launch human MC01, no native request sent',async()=>{
  const calls=[];
  const h=createHandler(config(),{native:async(...args)=>{calls.push(args);return snapshot(args[1]);}});
  const blocked=await h.handle({jsonrpc:'2.0',id:1,method:'tools/call',params:{name:'launch_profile',arguments:{slot:1}}});
  assert.equal(blocked.result.isError,true);
  assert.equal(calls.length,0);
  const launch=await h.handle({jsonrpc:'2.0',id:2,method:'tools/call',params:{name:'launch_profile',arguments:{slot:2}}});
  assert.equal(launch.result.isError,false);
  assert.deepEqual(calls.map(x=>x.slice(1)),[['launch',2]]);
  const full=createHandler(config('five-ai'),{native:async(...args)=>{calls.push(args);return snapshot(args[1]);}});
  assert.equal((await full.callTool('launch_profile',{slot:1})).operation,'launch');
});

test('observe_client emits a bounded MCP image only after exact identity recheck',async()=>{
  const calls=[];
  let same=true;
  const native=async(_exe,op,slot)=>{
    calls.push(['native',op,slot]);
    return {schema:1,ok:true,operation:'target',slot,pid:400+slot,
      windowId:same?1000+slot:9000+slot,
      createdFileTime:'134000000000000001',sessionId:1,
      foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'};
  };
  const capture=async(exe,target)=>{
    calls.push(['capture',exe,target.pid]);
    return {mimeType:'image/png',imageBase64:'iVBORw0KGgo=',
      width:800,height:600,sha256:'f'.repeat(64)};
  };
  const cfg={...config(),cuaExecutable:path.resolve('cua-driver.exe')};
  const h=createHandler(cfg,{native,capture});
  await assert.rejects(()=>h.callTool('observe_client',{slot:1}),/human host/);
  assert.equal(calls.length,0);
  const response=await h.handle({jsonrpc:'2.0',id:11,method:'tools/call',
    params:{name:'observe_client',arguments:{slot:2}}});
  assert.equal(response.result.isError,false);
  assert.equal(response.result.content[0].type,'text');
  assert.equal(response.result.content[1].type,'image');
  assert.equal(response.result.content[1].mimeType,'image/png');
  const metadata=JSON.parse(response.result.content[0].text);
  assert.equal(metadata.background,true);
  assert.equal(metadata.sourceTimeKind,'CUA_QUERY_STARTED');
  assert.equal(metadata.f3VisualReview.status,'WINDOW_FRAME_FOR_EXPLICIT_VISUAL_REVIEW_ONLY');
  assert.equal(metadata.f3VisualReview.coordinateExtraction,'NOT_PERFORMED');
  assert.equal(metadata.f3VisualReview.playerWorldAttribution,'NOT_VERIFIED');
  assert.equal(metadata.f3VisualReview.automaticCopyAllowed,false);
  assert.ok(Number.isSafeInteger(metadata.captureStartedAtMs));
  assert.ok(metadata.captureStartedAtMs<=Date.now());
  assert.deepEqual(calls.map(x=>x[0]),['native','capture','native']);
  same=false;
  // Reuse one capture stub but change the HWND on the second native check.
  let reads=0;
  const changed=createHandler(cfg,{native:async(...args)=>{
    reads++;same=reads===1;return native(...args);
  },capture});
  const rejected=await changed.handle({jsonrpc:'2.0',id:12,method:'tools/call',
    params:{name:'observe_client',arguments:{slot:2}}});
  assert.equal(rejected.result.isError,true);
  assert.equal(rejected.result.content.some(c=>c.type==='image'),false);
});

test('MCP local visual sampler returns only changed cached frames on demand',async()=>{
  const calls=[];
  const target=(slot)=>({schema:1,ok:true,operation:'target',slot,
    pid:500+slot,windowId:1200+slot,createdFileTime:'134000000000000001',
    sessionId:1,foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'});
  const configForSampler={...config(),cuaExecutable:path.resolve('cua-driver.exe')};
  const handler=createHandler(configForSampler,{
    native:async(_exe,operation,slot)=>{calls.push(['native',slot]);return target(slot);},
    capture:async(_exe,value)=>({mimeType:'image/png',imageBase64:'a'.repeat(64),
      width:800,height:600,sha256:'a'.repeat(64)}),
  });
  await assert.rejects(()=>handler.callTool('start_visual_observation',
    {slots:[1,2]}),/human host/);
  assert.equal(calls.length,0);
  const ready=await handler.callTool('start_visual_observation',
    {slots:[2],intervalMs:1250,durationMs:5000});
  assert.equal(ready.liveVideo,false);
  const pending=await handler.callTool('read_visual_observation',{slot:2});
  assert.equal(pending.metadata.status,'WAITING_FOR_FIRST_FRAME');
  assert.equal(pending.metadata.f3VisualReview.status,'NO_CURRENT_WINDOW_FRAME');
  assert.equal(pending.metadata.f3VisualReview.automaticCopyAllowed,false);
  assert.equal(pending.metadata.screenReviewFrameDelivered,false);
  const reviewed=await handler.callTool('read_visual_observation',{
    slot:2,includeImage:false,screenReview:{frameSequence:1,sourceFrameAtMs:1000,
      identity:{pid:502,windowId:1202,createdFileTime:'134000000000000001',sessionId:1},
      recognizedLine:'Position: 12, 64, -8'},
  });
  assert.equal(reviewed.imageBase64,null);
  assert.equal(reviewed.metadata.f3ScreenXYZ.status,'SCREEN_FRAME_STALE_OR_REPLACED');
  assert.equal(reviewed.metadata.f3ScreenXYZ.automaticCopyAllowed,false);
  await assert.rejects(()=>handler.callTool('read_visual_observation',{
    slot:2,screenReview:{frameSequence:1,sourceFrameAtMs:1000,
      identity:{pid:502,windowId:1202,createdFileTime:'134000000000000001',sessionId:1},
      recognizedLine:'Position: 12, 64, -8'},
  }),/includeImage=false/);
  const stopped=await handler.callTool('stop_visual_observation',{slots:[2]});
  assert.equal(stopped.captureFramesRetained,false);
  handler.shutdown();
});

test('opt-in persistent GPU observation reuses MCP tools and protects MC01',async()=>{
  const calls=[];
  const latestFrame={provider:'NATIVE_WGC_PERSISTENT',
    mimeType:'image/png',imageBase64:'a'.repeat(64),width:960,height:540,
    sha256:'a'.repeat(64),capturedAtMs:Date.now()-25,workerSequence:7,
    frameFreshness:'CAPTURE_TIMESTAMP_RECENT_GAMEPLAY_NOT_VERIFIED'};
  const gpuFactory=()=>({
    async start(slot){calls.push(['start',slot]);return {slot};},
    latest(slot,binding){calls.push(['latest',slot,binding]);return latestFrame;},
    stop(slots){calls.push(['stop',...slots]);},
    shutdown(){calls.push(['shutdown']);},
  });
  const target=slot=>({schema:1,ok:true,operation:'target',
    slot,pid:600+slot,windowId:3000+slot,
    createdFileTime:'134000000000000001',sessionId:1,
    foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'});
  const cfg={...config(),gpuExecutable:path.resolve('lazy-tester-gpu-capture.exe')};
  const handler=createHandler(cfg,{
    gpuSourceFactory:gpuFactory,
    native:async(_exe,operation,slot)=>{calls.push(['native',slot]);return target(slot);},
  });
  await assert.rejects(()=>handler.callTool('start_visual_observation',
    {slots:[1]}),/human host/);
  assert.equal(calls.length,0);
  const started=await handler.callTool('start_visual_observation',
    {slots:[2],durationMs:5000});
  assert.equal(started.source,'PERSISTENT_NATIVE_WGC_EXPLICIT_OPT_IN');
  const frame=await handler.callTool('observe_client',{slot:2});
  assert.equal(frame.metadata.provider,'NATIVE_WGC_PERSISTENT');
  assert.equal(frame.metadata.background,true);
  assert.equal(frame.metadata.sourceTimeKind,'WGC_READBACK_COMPLETED');
  assert.equal(frame.metadata.capturedAtMs,latestFrame.capturedAtMs);
  assert.equal(frame.metadata.workerSequence,7);
  assert.equal(frame.metadata.f3VisualReview.status,'WINDOW_FRAME_FOR_EXPLICIT_VISUAL_REVIEW_ONLY');
  assert.equal(frame.metadata.f3VisualReview.playerWorldAttribution,'NOT_VERIFIED');
  assert.equal(frame.metadata.f3VisualReview.automaticCopyAllowed,false);
  const boundRead=calls.find(([operation])=>operation==='latest');
  assert.equal(boundRead[1],2);
  assert.equal(boundRead[2].pid,602);
  assert.equal(boundRead[2].windowId,3002);
  assert.equal(boundRead[2].createdFileTime,'134000000000000001');
  assert.equal(boundRead[2].sessionId,1);
  handler.callTool('stop_visual_observation',{slots:[2]});
  handler.shutdown();
  assert.equal(calls.some(([op])=>op==='start'),true);
  assert.equal(calls.some(([op])=>op==='stop'),true);
  assert.equal(calls.some(([op])=>op==='shutdown'),true);
});

test('observe_client refuses an unconfigured provider without calling native',async()=>{
  let called=0;
  const h=createHandler(config(),{native:async()=>{called++;throw Error('unexpected');}});
  await assert.rejects(()=>h.callTool('observe_client',{slot:2}),/NOT_CONFIGURED/);
  assert.equal(called,0);
});

test('AI computer-use target resolves exact window without touching human MC01',async()=>{
  const calls=[];
  const native=async(_exe,op,slot)=>{
    calls.push([op,slot]);
    return {schema:1,ok:true,operation:'target',slot,pid:240+slot,
      windowId:1000+slot,createdFileTime:'134000000000000001',
      sessionId:1,foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'};
  };
  const h=createHandler(config(),{native});
  await assert.rejects(()=>h.callTool('get_computer_use_target',{slot:1}),
    /human host is protected/);
  assert.equal(calls.length,0);
  const result=await h.callTool('get_computer_use_target',{slot:2});
  assert.deepEqual(result.target,{kind:'window',pid:242,window_id:1002});
  assert.equal(result.capture,'NOT_CAPTURED_BY_LAZY_TESTER');
  assert.equal(result.gameInput,'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED');
  assert.deepEqual(calls,[['target',2]]);
  const exact={slot:2,pid:242,windowId:1002,
    createdFileTime:'134000000000000001'};
  assert.equal((await h.callTool('revalidate_computer_use_target',exact)).matched,true);
  assert.equal((await h.callTool('revalidate_computer_use_target',
    {...exact,windowId:1003})).status,'TARGET_CHANGED_REFUSE_ACTION');
  assert.deepEqual(calls,[['target',2],['target',2],['target',2]]);
});

test('native target contract rejects forged, stale and non-window observations',()=>{
  const base={schema:1,ok:true,operation:'target',slot:2,
    pid:42,windowId:1002,createdFileTime:'134000000000000001',
    sessionId:1,foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'};
  assert.equal(validateNativeWindowTarget(base,2).windowId,1002);
  for(const invalid of [
    {...base,slot:1},{...base,pid:0},
    {...base,windowId:Number.MAX_SAFE_INTEGER+1},
    {...base,createdFileTime:'not-a-filetime'},
    {...base,visible:false},{...base,minimized:true},
    {...base,verification:'PID_ONLY'},
    {...base,gameplayInput:'VERIFIED'},
  ]) assert.throws(()=>validateNativeWindowTarget(invalid,2));
});

test('MCP peer-session guidance inspects once and never claims world join',async()=>{
  const calls=[];
  const handler=createHandler(config(),{native:async(_exe,op)=>{calls.push(op);return snapshot(op);}});
  const plan=await handler.callTool('plan_peer_session',{});
  assert.deepEqual(calls,['inspect']);
  assert.equal(plan.hostWorld,'NOT_OBSERVED');
  assert.equal(plan.independentBackgroundInput,'NOT_VERIFIED');
});

test('MCP compiles simultaneous controls without dispatching native input',async()=>{
  let nativeCalls=0;
  const h=createHandler(config(),{native:async()=>{
    nativeCalls++;throw Error('Should never touch native');
  }});
  const result=await h.callTool('plan_player_actions',{includeTimeline:true,actions:[
    {slot:2,kind:'move_forward',atMs:0,durationMs:300},
    {slot:2,kind:'attack',atMs:0,durationMs:100},
    {slot:2,kind:'look_relative',atMs:0,durationMs:50,deltaX:60,deltaY:-5},
  ]});
  assert.equal(result.execution,'NOT_DISPATCHED');
  assert.equal(result.maximumConcurrentIntents,3);
  assert.equal(result.inputEvents.filter(e=>e.phase==='PRESS').length,2);
  assert.equal(result.inputEvents.filter(e=>e.phase==='POINTER_DELTA').length,1);
  assert.match(result.deliveryGate,/BLOCKED/);
  assert.equal(nativeCalls,0);
});

test('native input route assessment only observes trusted windows and blocks delivery',async()=>{
  const calls=[];
  const h=createHandler(config(),{native:async(_exe,op,slot)=>{
    calls.push([op,slot]);
    return {schema:1,ok:true,operation:'target',slot,
      pid:500+slot,windowId:2000+slot,
      createdFileTime:'134000000000000001',sessionId:1,
      foreground:slot===1,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED'};
  }});
  const result=await h.callTool('assess_input_route',{actions:[
    {slot:2,kind:'move_forward',atMs:0,durationMs:200},
    {slot:3,kind:'look_relative',atMs:0,durationMs:50,
      deltaX:20,deltaY:-5},
  ]});
  assert.deepEqual(calls,[['target',1],['target',2],['target',3]]);
  assert.deepEqual(result.aiTargets.map(x=>x.slot),[2,3]);
  assert.equal(result.execution,'NOT_DISPATCHED');
  assert.equal(result.gameplayInputAcceptance,'NOT_EXECUTED');
  assert.equal(result.transactionEngine.execution,'DISABLED');
  assert.equal(result.transactionEngine.provider,'NONE');
  assert.match(result.deliveryGate,/BLOCKED/);
  assert.equal(result.aiTargets.some(x=>x.slot===1),false);
  await assert.rejects(()=>h.callTool('assess_input_route',{actions:[
    {slot:1,kind:'attack',atMs:0,durationMs:100}
  ]}),/human-owned/);
  assert.equal(calls.length,3);
  h.shutdown();
});

test('native input route refuses aliasing between two Minecraft identities',async()=>{
  const h=createHandler(config(),{native:async(_exe,op,slot)=>({
    schema:1,ok:true,operation:'target',slot,
    pid:slot===3?502:500+slot,windowId:2000+slot,
    createdFileTime:'134000000000000001',sessionId:1,
    foreground:slot===1,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
    gameplayInput:'NOT_VERIFIED'
  })});
  await assert.rejects(()=>h.callTool('assess_input_route',{actions:[
    {slot:2,kind:'move_forward',atMs:0,durationMs:100},
    {slot:3,kind:'attack',atMs:0,durationMs:100},
  ]}),/share one native/);
  h.shutdown();
});

test('MCP player-action plan is nonexecuting and protects MC01',async()=>{
  const calls=[];
  const handler=createHandler(config(),{native:async(...args)=>{calls.push(args);return snapshot(args[1]);}});
  const response=await handler.callTool('plan_player_actions',{actions:[
    {slot:2,kind:'move_forward',atMs:0,durationMs:100},
    {slot:3,kind:'jump',atMs:0,durationMs:100}
  ]});
  assert.equal(response.execution,'NOT_DISPATCHED');
  assert.match(response.gate,/BLOCKED/);
  assert.equal(calls.length,0);
  await assert.rejects(()=>handler.callTool('plan_player_actions',{actions:[
    {slot:1,kind:'jump',atMs:0,durationMs:100}]}),/human-owned/);
});

test('native operations serialized and malformed slot denied',async()=>{
  let release, count=0;
  const native=async(_exe,operation)=>{count++;await new Promise(r=>{release=r});return snapshot(operation);};
  const h=createHandler(config(),{native});
  const first=h.callTool('inspect_profiles',{});
  await assert.rejects(()=>h.callTool('launch_profile',{slot:2}),/in progress/);
  await assert.rejects(()=>h.callTool('launch_profile',{slot:3,command:'del files'}),/Invalid tool arguments/);
  await assert.rejects(()=>h.callTool('launch_profile',{slot:6}),/slot must/);
  assert.equal(count,1);
  release();assert.equal((await first).operation,'inspect');
});

test('one read-only context handoff combines current profiles and bounded game evidence',async()=>{
  const calls=[];
  const handler=createHandler(config(),{
    native:async(_exe,op)=>{calls.push(op);return snapshot(op);},
    evidence:async(_directory,slot)=>({
      slot,status:slot===2?'CAPTURE_FILE_PRESENT':'NO_CAPTURE_FILE',
      events:slot===2?[{observedAt:'2026-10-30T12:30:00Z',level:'log',
        type:'game_results',data:{schema:1,record_type:'gameplay_result',total_score:8,
          data:{canonical:{record_id:'r1'}}}}]:[],
    }),
  });
  const context=await handler.callTool('get_test_context',{slots:[2]});
  assert.deepEqual(calls,['inspect']);
  assert.equal(context.hostSlot,1);
  assert.equal(context.nativeProfiles.length,5);
  assert.equal(context.peerPlan.hostWorld,'NOT_OBSERVED');
  assert.equal(context.gameEvidence.resultCount,1);
  assert.equal(context.gameControl,'INDEPENDENT_BACKGROUND_INPUT_NOT_VERIFIED');
  assert.equal(context.contextScope,'NATIVE_SESSION_AND_UNATTRIBUTED_PROBE_EVIDENCE_ONLY');
});

test('MCP summarized telemetry never interprets capture filenames as verified identity',async()=>{
  const handler=createHandler(config(),{evidence:async(_directory,slot)=>({
    slot,status:slot===3?'CAPTURE_FILE_PRESENT':'NO_CAPTURE_FILE',
    events:slot===3?[{observedAt:'2026-10-30T13:00:00Z',level:'log',
      type:'analytics',data:{type:'death',session_id:'s1',record_id:'d1'}}]:[],
    attribution:'OPERATOR_NAMED_UNVERIFIED'
  })});
  const report=await handler.callTool('summarize_test_evidence',{slots:[3,1]});
  assert.deepEqual(report.requestedSlots,[1,3]);
  assert.equal(report.worldIdentity,'NOT_VERIFIED');
  assert.equal(report.attentionCount,1);
  assert.deepEqual(report.missingSlots,[1]);
  await assert.rejects(()=>handler.callTool('summarize_test_evidence',{slots:[2,2]}),
    /distinct/);
});

test('evidence reads only complete marked lines from assigned profile',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-agent-'));
  try {
    const log=['2026-10-30T12:00:00.000Z log unrelated',
      '2026-10-30T12:00:01.000Z warn [ANALYTICS]{"schema":1,"type":"stage_attempt","value":2}',
      '2026-10-30T12:00:02.000Z log [GAME_RESULTS]{"schema":1,"record_type":"gameplay_result","total_score":8}',
      '2026-10-30T12:00:03.000Z warn [ANALYTICS]{"partial":true}'
    ].join('\n');
    await writeFile(path.join(root,'MC02.log'),log);
    const seen=await readGameEvidence(root,2);
    assert.equal(seen.status,'CAPTURE_FILE_PRESENT');assert.equal(seen.events.length,2);
    assert.deepEqual(seen.events.map(x=>x.type),['analytics','game_results']);
    assert.equal(seen.attribution,'OPERATOR_NAMED_UNVERIFIED');
    assert.equal((await readGameEvidence(root,3)).status,'NO_CAPTURE_FILE');
    assert.equal(parseCapturedLine('2026-10-30T12:00:01.000Z warn hello'),null);
    const encoded=JSON.stringify('[ANALYTICS] {"schema":1,"type":"death"}\n\r').slice(1,-1);
    assert.equal(parseCapturedLine('2026-10-30T12:00:04.000Z log '+encoded).data.type,'death');
    await symlink(path.join(root,'MC02.log'),path.join(root,'MC03.log'));
    await assert.rejects(()=>readGameEvidence(root,3),/ordinary file/);
  } finally {await rm(root,{recursive:true,force:true});}
});

test('Probe QA cursor accepts only appended complete lines from same capture',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-probe-cursor-'));
  const filename=path.join(root,'MC02.log');
  const row=(issue,fields,at='2026-11-02T08:00:00Z')=>
    at+' warn [probe:probe] INFO tick=120 issue='+issue+' '+fields+'\n';
  try {
    // Historic complete Probe markers must not be read by a new plan.
    await writeFile(filename,
      row('RUN_START','arg=auto count=1')+
      row('RESULT','id=P01 result=pass build=dev detail=historic')+
      row('RUN_DONE','arg=auto count=1'));
    const prepared=await readGameEvidence(root,2,20,{probeOnly:true});
    assert.equal(prepared.status,'CAPTURE_FILE_PRESENT');
    assert.equal(prepared.captureCursor.kind,'STABLE_CAPTURE_CURSOR_V1');
    assert.deepEqual(prepared.probeEvents,[]);
    assert.deepEqual(prepared.events,[]); // QA never parses unrelated records

    const waiting=await readGameEvidence(root,2,20,{
      probeCheckpoint:prepared.captureCursor});
    assert.deepEqual(waiting.probeEvents,[]);

    await appendFile(filename,
      row('RUN_START','arg=auto count=2','2026-11-02T08:01:00Z')+
      row('RESULT','id=P01 result=pass build=dev detail=new','2026-11-02T08:01:01Z'));
    const partial=await readGameEvidence(root,2,20,{
      probeCheckpoint:prepared.captureCursor});
    assert.deepEqual(partial.probeEvents.map(x=>x.kind),['start','result']);

    await appendFile(filename,
      row('RESULT','id=P02 result=fail build=dev detail=new','2026-11-02T08:01:02Z')+
      row('RUN_DONE','arg=auto count=2','2026-11-02T08:01:03Z'));
    const completed=await readGameEvidence(root,2,20,{
      probeCheckpoint:prepared.captureCursor});
    assert.deepEqual(completed.probeEvents.map(x=>x.kind),
      ['start','result','result','done']);
    assert.equal(completed.probeEvents[2].result,'fail');
    assert.equal(completed.captureCursor.byteOffset>prepared.captureCursor.byteOffset,true);

    // Replacing the physical file, even at the same pathname, invalidates
    // the checkpoint; old bytes are not accepted as a new QA run.
    await rename(filename,path.join(root,'old.log'));
    await writeFile(filename,row('RUN_START','arg=auto count=2'));
    const replaced=await readGameEvidence(root,2,20,{
      probeCheckpoint:prepared.captureCursor});
    assert.equal(replaced.status,'CAPTURE_FILE_UNSTABLE');
    assert.equal(replaced.reason,'PROBE_CAPTURE_REPLACED');
  } finally {await rm(root,{recursive:true,force:true});}
});

test('raw Skills Probe debugger lines remain valid without invented timestamps',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-probe-raw-'));
  const filename=path.join(root,'MC02.log');
  try {
    await writeFile(filename,'Debugger attached to [127.0.0.1] on port [19144].\n');
    const before=await readGameEvidence(root,2);
    await appendFile(filename,
      '[probe:probe] INFO tick=1000 issue=RUN_START arg=auto count=1\n'+
      '[probe:probe] INFO tick=1002 issue=RESULT id=P01 result=pass build=dev detail=okay\n'+
      '[probe:probe] INFO tick=1002 issue=RUN_DONE arg=auto count=1\n');
    const after=await readGameEvidence(root,2,20,{
      probeCheckpoint:before.captureCursor});
    assert.deepEqual(after.probeEvents.map(e=>e.kind),
      ['start','result','done']);
    assert.equal(after.probeEvents[1].at,null);
  } finally {await rm(root,{recursive:true,force:true});}
});

test('Probe QA cursor rejects file rewrites, truncated tails and partial lines',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-probe-rewrite-'));
  const filename=path.join(root,'MC02.log');
  const prefix='2026-11-02T08:00:00Z warn starting captured content\n';
  try {
    await writeFile(filename,prefix);
    const first=await readGameEvidence(root,2);
    await appendFile(filename,
      '2026-11-02T08:01:00Z warn [probe:probe] INFO tick=120 issue=RUN_START arg=auto count=1');
    let observed=await readGameEvidence(root,2,20,{probeCheckpoint:first.captureCursor});
    assert.deepEqual(observed.probeEvents,[]); // incomplete line
    await appendFile(filename,'\n');
    observed=await readGameEvidence(root,2,20,{probeCheckpoint:first.captureCursor});
    assert.equal(observed.probeEvents[0].kind,'start');

    // Modify checkpoint boundary in the same file without replacing inode.
    await writeFile(filename,prefix.replace('starting','modified')+
      '2026-11-02T08:02:00Z warn [probe:probe] INFO tick=121 issue=RUN_START arg=auto count=1\n');
    observed=await readGameEvidence(root,2,20,{probeCheckpoint:first.captureCursor});
    assert.equal(observed.status,'CAPTURE_FILE_UNSTABLE');
    assert.equal(observed.reason,'PROBE_CAPTURE_PREFIX_CHANGED');
  } finally {await rm(root,{recursive:true,force:true});}
});

test('experimental background input is NOT exposed as an agent tool',async()=>{
  const h=createHandler(config());
  const tools=(await h.handle({jsonrpc:'2.0',id:1,method:'tools/list'})).result.tools;
  // Read-only assessment is expected; no actuating input tool is allowed.
  assert.equal(tools.some(t=>['send_input','inject_input',
    'execute_game_input','probe-background'].includes(t.name)),false);
  assert.equal(tools.some(t=>t.name==='assess_input_route'),true);
  assert.equal((await h.handle({jsonrpc:'2.0',id:2,method:'tools/call',
    params:{name:'probe-background',arguments:{slot:2}}})).result.isError,true);
  const capabilities=await h.callTool('agent_capabilities',{});
  assert.equal(capabilities.notAvailable.includes(
    'parallel_independent_keyboard_mouse'),true);
  assert.equal(capabilities.independentInput.nativeSessionModel,
    'SAME_WINDOWS_SESSION_ONLY');
  assert.equal(capabilities.independentInput.windowsAgentWorkspace,
    'PREVIEW_NOT_INTEGRATED_NO_CROSS_SESSION_INPUT');
  assert.equal(capabilities.activity.displayHint,'HIDDEN_NO_LOCAL_SIGNAL');
  assert.equal(capabilities.activity.desktopControl,
    'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
});


test('publishing an activity summary is opt-in, bounded, and never game control',async()=>{
  let publishes=0,seen=null;
  const handler=createHandler(config(),{
    native:async()=>{throw Error('No native game/QA request allowed');},
    publishActivity:async(_exe,status)=>{
      publishes++;seen=status;
      return {status:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER'};
    },
  });
  const defaultResult=await handler.callTool('agent_capabilities',{});
  assert.equal(publishes,0,'ordinary capabilities reads never contact native');
  assert.equal(defaultResult.activity.displayHint,'HIDDEN_NO_LOCAL_SIGNAL');
  const published=await handler.callTool('agent_capabilities',{publishToDesktop:true});
  assert.equal(publishes,1);
  assert.equal(published.desktopDelivery,'REPORTED_TO_EXISTING_NATIVE_CONTROLLER');
  assert.equal(seen.desktopControl,'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
  assert.equal(seen.minecraftInputEffect,'NOT_VERIFIED');
  assert.equal(seen.mapGameplayAcceptance,'NOT_TESTED');
  await assert.rejects(()=>handler.callTool('agent_capabilities',{
    publishToDesktop:'true'}),/Invalid tool arguments/);
  const offline=createHandler(config(),{
    publishActivity:async()=>{throw Error('No retained controller');},
  });
  const unavailable=await offline.callTool('agent_capabilities',{publishToDesktop:true});
  assert.equal(unavailable.desktopDelivery,'STATUS_DELIVERY_UNAVAILABLE');
  assert.equal(unavailable.activity.displayHint,'HIDDEN_NO_LOCAL_SIGNAL');
  handler.shutdown();offline.shutdown();
});


test('one probe_qa call can optionally mirror the SAME source-only Testing state',async()=>{
  let catalogs=0,captures=0,published=0,mutableCalls=[];
  const fixture=createHandler({...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    native:async(_exe,operation,slot)=>{
      mutableCalls.push([operation,slot]);
      assert.equal(operation,'target','Only read-only identity may be inspected');
      assert.equal(slot,2);
      return {schema:1,ok:true,operation:'target',slot,
        pid:402,windowId:802,createdFileTime:'99001',sessionId:1,
        foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
    probeCatalog:async()=>{
      catalogs++;
      return {autoCount:2,autoIds:['P01','P02'],
        archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
        sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=> 'UNCHANGED',
    evidence:async(_root,slot,_limit,options)=>{
      captures++;
      if(options?.probeCheckpoint) {
        return {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:2,tick:100,at:null},
          {kind:'result',id:'P01',result:'pass',tick:101,at:null},
          {kind:'result',id:'P02',result:'pass',tick:102,at:null},
          {kind:'done',arg:'auto',count:2,tick:103,at:null},
        ]};
      }
      return {slot,status:'CAPTURE_FILE_PRESENT',captureCursor:{
        kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
    publishActivity:async(_exe,activity)=>{
      published++;
      assert.equal(activity.desktopControl,
        'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER');
      assert.equal(activity.mapGameplayAcceptance,'NOT_TESTED');
      assert.equal(activity.items[0]?.slot,2);
      return {status:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER'};
    },
  });
  const first=await fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:true});
  assert.equal(first.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(first.operatorNextAction,
    'AWAIT_AUTHORIZED_PLAYER_COMMAND_AND_QA_WORLD');
  assert.equal(first.commandSentByLazyTester,false);
  assert.equal(first.desktopDelivery,'REPORTED_TO_EXISTING_NATIVE_CONTROLLER');
  const second=await fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:true});
  assert.equal(second.status,'PROBE_REPORTED_PASS');
  assert.equal(second.terminal,true);
  assert.equal(second.operatorNextAction,
    'REVIEW_REPORTED_PROBE_PASS_MAP_NOT_TESTED');
  assert.equal(second.counts.pass,2);
  assert.equal(second.desktopDelivery,'REPORTED_TO_EXISTING_NATIVE_CONTROLLER');
  const beforeRepeated=[catalogs,captures,mutableCalls.length];
  const duplicate=await fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:true});
  assert.equal(duplicate.status,second.status);
  assert.equal(duplicate.counts.pass,second.counts.pass);
  assert.deepEqual([catalogs,captures,mutableCalls.length],beforeRepeated,
    'Mirroring a terminal result must not rerun source, log or native checks');
  assert.equal(published,3,'One explicit mirror per opted-in call only');
  const legacy=await fixture.callTool('probe_qa',{slot:2});
  assert.equal(legacy.operatorNextAction,undefined);
  assert.equal(legacy.desktopDelivery,undefined);
  assert.equal(published,3,'Legacy read does not publish anything');
  const hintOnly=await fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:false});
  assert.equal(hintOnly.operatorNextAction,
    'REVIEW_REPORTED_PROBE_PASS_MAP_NOT_TESTED');
  assert.equal(hintOnly.desktopDelivery,undefined);
  assert.equal(published,3);
  await assert.rejects(()=>fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:'true'}),/Invalid tool arguments/);
  assert.equal(catalogs,1,'One checkpoint was prepared');
  fixture.shutdown();
});

test('AI one-entry preflight refusal cannot mirror an older or invented run',async()=>{
  let published=0;
  const blocked=createHandler(config(),{
    native:async()=>{throw Error('No native call authorized');},
    publishActivity:async()=>{published++;return {
      status:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER'};},
  });
  const result=await blocked.callTool('probe_qa',
    {slot:2,publishToDesktop:true});
  assert.equal(result.phase,'PREFLIGHT');
  assert.equal(result.status,'BLOCKED');
  assert.equal(result.operatorNextAction,'RESOLVE_APPROVED_PROBE_SOURCE_OR_CAPTURE');
  assert.equal(result.desktopDelivery,
    'NO_CURRENT_TESTING_CHECKPOINT_TO_MIRROR');
  assert.equal(published,0,'Do not falsely publish an unauthenticated run');
  await assert.rejects(()=>blocked.callTool('probe_qa',
    {slot:1,publishToDesktop:true}),/MC01 human host is protected/);
  blocked.shutdown();
});

test('operator hints distinguish waiting, source-only, and blocked terminal reports',async()=>{
  const fixture=createHandler(config());
  assert.equal((await fixture.callTool('probe_qa',
    {slot:2,publishToDesktop:false})).operatorNextAction,
    'RESOLVE_APPROVED_PROBE_SOURCE_OR_CAPTURE');
  fixture.shutdown();
});

test('native-coordinated Testing admits one MCP process and rejects a rival',async()=>{
  const nativeRuns=new Map();
  const calls=[];
  const coordinate=async(_exe,request)=>{
    calls.push({...request});
    let existing=nativeRuns.get(request.slot);
    if(request.operation==='reserve') {
      if(existing&&request.replaceRunId===existing.runId &&
         existing.phase==='FINAL'&&existing.ownerId===request.ownerId) {
        existing=null;
      } else if(existing) {
        return {schema:1,slot:request.slot,runId:existing.runId,
          phase:existing.phase,
          ownerMatch:existing.ownerId===request.ownerId &&
            existing.runId===request.runId,
          reportedStatus:existing.reportedStatus,
          reportDigest:existing.reportDigest,
          provenance:'NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'};
      }
      const run={schema:1,slot:request.slot,runId:request.runId,
        ownerId:request.ownerId,phase:'RESERVED',ownerMatch:true,
        reportedStatus:null,reportDigest:null,
        provenance:'NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'};
      nativeRuns.set(request.slot,run);
      return {...run};
    }
    assert.equal(request.operation,'finalize');
    assert.equal(existing.runId,request.runId);
    assert.equal(existing.ownerId,request.ownerId);
    if(existing.phase==='FINAL') {
      assert.equal(existing.reportDigest,request.reportDigest);
      assert.equal(existing.reportedStatus,request.reportedStatus);
    } else {
      existing.phase='FINAL';
      existing.reportedStatus=request.reportedStatus;
      existing.reportDigest=request.reportDigest;
    }
    return {...existing};
  };
  let catalogs=0,captures=0,published=0;
  const configA={...config(),probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')};
  const shared={
    coordinateTesting:coordinate,
    publishActivity:async(_exe,activity)=>{
      published++;
      assert.equal(activity.mapGameplayAcceptance,'NOT_TESTED');
      return {status:'REPORTED_TO_EXISTING_NATIVE_CONTROLLER'};
    },
    probeCatalog:async()=>{
      catalogs++;
      return {autoCount:1,autoIds:['P01'],
        archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
        sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=> 'UNCHANGED',
    native:async(_exe,operation,slot)=>({
      schema:1,ok:true,operation:'target',slot,pid:402,windowId:802,
      createdFileTime:'9001',sessionId:1,foreground:false,
      visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED',
    }),
    evidence:async(_path,slot,_n,opts)=>{
      captures++;
      return opts?.probeCheckpoint?
        {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
          {kind:'start',arg:'auto',count:1,tick:1,at:null},
          {kind:'result',id:'P01',result:'pass',tick:2,at:null},
          {kind:'done',arg:'auto',count:1,tick:3,at:null},
        ]}:
        {slot,status:'CAPTURE_FILE_PRESENT',
          captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  };
  const one=createHandler(configA,shared),two=createHandler(configA,shared);
  const prepared=await one.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(prepared.phase,'AWAITING_AUTHORIZED_COMMAND');
  assert.equal(prepared.nativeRunStatus,'RESERVED');
  assert.equal(prepared.desktopDelivery,'REPORTED_TO_EXISTING_NATIVE_CONTROLLER');
  assert.match(prepared.testingRunId,/^[a-f0-9]{32}$/);
  const denied=await two.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(denied.status,'BLOCKED');
  assert.equal(denied.reason,'TESTING_RUN_ALREADY_RESERVED');
  assert.equal(denied.existingRunId,prepared.testingRunId);
  assert.equal(catalogs,1,'A competing MCP cannot prepare duplicate source Testing');
  const completed=await one.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(completed.status,'PROBE_REPORTED_PASS');
  assert.equal(completed.nativeRunStatus,'FINAL');
  assert.match(completed.reportDigest,/^[a-f0-9]{64}$/);
  const snap=[catalogs,captures];
  const reused=await one.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(reused.reportDigest,completed.reportDigest);
  assert.deepEqual([catalogs,captures],snap);
  const rivalAgain=await two.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(rivalAgain.reason,'TESTING_RUN_ALREADY_RESERVED');
  assert.equal(published,3,'Only owner publishes source Testing summaries');
  const restart=await one.callTool('probe_qa',
    {slot:2,coordinateNative:true,restart:true,publishToDesktop:true});
  assert.equal(restart.nativeRunStatus,'RESERVED');
  assert.notEqual(restart.testingRunId,prepared.testingRunId);
  assert.equal(catalogs,2,'An explicit owner restart creates one new checkpoint');
  // Legacy direct preparation invalidates only this MCP's local checkpoint
  // identity. It cannot silently inherit the old native reservation.
  const legacy=await one.callTool('prepare_probe_qa',{slot:2});
  assert.equal(legacy.status,'PREPARED_SOURCE_ONLY');
  const mismatched=await one.callTool('probe_qa',
    {slot:2,coordinateNative:true});
  assert.equal(mismatched.reason,
    'EXISTING_LEGACY_TESTING_CHECKPOINT_NOT_COORDINATED');
  one.shutdown();two.shutdown();
});

test('native coordination fails closed without creating a local Testing checkpoint',async()=>{
  const offline=createHandler({...config(),executable:''},{
    coordinateTesting:async()=>{throw Error('No controller allowed');},
  });
  const blocked=await offline.callTool('probe_qa',
    {slot:2,coordinateNative:true,publishToDesktop:true});
  assert.equal(blocked.reason,'TESTING_NATIVE_CONTROLLER_NOT_CONFIGURED');
  const lost=createHandler(config(),{
    coordinateTesting:async()=>{throw Error('Native status unavailable');},
    native:async()=>{throw Error('Must not request native Minecraft');},
  });
  assert.equal((await lost.callTool('probe_qa',
    {slot:2,coordinateNative:true})).reason,
    'TESTING_NATIVE_RESERVATION_UNCONFIRMED');
  await assert.rejects(()=>lost.callTool('probe_qa',
    {slot:1,coordinateNative:true}),/MC01 human host is protected/);
  await assert.rejects(()=>lost.callTool('probe_qa',
    {slot:2,coordinateNative:'true'}),/Invalid tool arguments/);
  offline.shutdown();lost.shutdown();
});

test('native Testing helper validates correlation before any subprocess',async()=>{
  await assert.rejects(()=>coordinateTestingRunNative('',{
    operation:'reserve',slot:2,runId:'a'.repeat(32),
    ownerId:'b'.repeat(32),
  }),/Native Testing controller not configured/);
  const executable=path.resolve('lazy-tester.exe');
  await assert.rejects(()=>coordinateTestingRunNative(executable,{
    operation:'reserve',slot:0,runId:'a'.repeat(32),
    ownerId:'b'.repeat(32),
  }),/Invalid Testing run correlation request/);
  await assert.rejects(()=>coordinateTestingRunNative(executable,{
    operation:'reserve',slot:2,runId:'A'.repeat(32),
    ownerId:'b'.repeat(32),
  }),/Invalid Testing run correlation request/);
  await assert.rejects(()=>coordinateTestingRunNative(executable,{
    operation:'finalize',slot:2,runId:'a'.repeat(32),
    ownerId:'b'.repeat(32),reportedStatus:'PASS',
    reportDigest:'c'.repeat(64),
  }),/Invalid Testing terminal reference/);
});


test('AI retrieves an archived original Probe summary after MCP restart without game IPC',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lt-mcp-testing-store-'));
  const data=path.join(root,'Data');await mkdir(data);
  const archive=path.join(data,'Testing');
  const cfg={...config(),testingArchiveDirectory:archive,
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')};
  let catalogs=0,reads=0,native=0,arbitration=0;
  const coordination=async(_exe,args)=>{
    arbitration++;
    if(args.operation==='reserve')return {
      schema:1,slot:args.slot,runId:args.runId,phase:'RESERVED',
      ownerMatch:true,reportedStatus:null,reportDigest:null,
      provenance:'NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'};
    return {schema:1,slot:args.slot,runId:args.runId,
      phase:'FINAL',ownerMatch:true,reportedStatus:args.reportedStatus,
      reportDigest:args.reportDigest,
      provenance:'NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'};
  };
  const setup={
    coordinateTesting:coordination,
    probeCatalog:async()=>{catalogs++;return {
      autoCount:2,autoIds:['P01','P02'],
      archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'};
    },
    probeRevisionStatus:async()=> 'UNCHANGED',
    native:async(_exe,operation,slot)=>{
      native++;
      return {schema:1,ok:true,operation:'target',slot,
        pid:4002,windowId:8002,createdFileTime:'9001',
        sessionId:1,foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
    evidence:async(_root,slot,_n,options)=>{
      reads++;
      if(options?.probeCheckpoint)return {
        slot,status:'CAPTURE_FILE_PRESENT',
        probeEvents:[
          {kind:'start',arg:'auto',count:2,tick:10,at:null},
          {kind:'result',id:'P01',result:'pass',tick:11,at:null},
          {kind:'result',id:'P02',result:'pass',tick:12,at:null},
          {kind:'done',arg:'auto',count:2,tick:13,at:null},
        ]};
      return {slot,status:'CAPTURE_FILE_PRESENT',
        captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}};
    },
  };
  const active=createHandler(cfg,setup);
  try {
    const prepared=await active.callTool('probe_qa',
      {slot:2,coordinateNative:true});
    assert.equal(prepared.phase,'AWAITING_AUTHORIZED_COMMAND');
    const finished=await active.callTool('probe_qa',
      {slot:2,coordinateNative:true});
    assert.equal(finished.status,'PROBE_REPORTED_PASS');
    assert.equal(finished.terminal,true);
    assert.equal(finished.archiveStatus,'ARCHIVED_SOURCE_ONLY');
    assert.equal(finished.archivedRunId,finished.testingRunId);
    assert.match(finished.reportDigest,/^[a-f0-9]{64}$/);
    const originalFile=await readFile(path.join(archive,'runs',
      finished.testingRunId,'report.json'),'utf8');
    assert.equal(originalFile.includes('pid'),false);
    assert.equal(originalFile.includes('windowId'),false);
    const readBefore=[catalogs,reads,native,arbitration];
    const again=await active.callTool('probe_qa',
      {slot:2,coordinateNative:true});
    assert.equal(again.archivedRunId,finished.testingRunId);
    assert.deepEqual([catalogs,reads,native],[...readBefore].slice(0,3));
    active.shutdown();

    // A fresh MCP has neither the former checkpoint nor native owner
    // lease. Its history read is explicit, read-only and source-limited.
    const restarted=createHandler(cfg,{
      native:async()=>{throw Error('Historical read must not contact Minecraft');},
      coordinateTesting:async()=>{throw Error('No new reservation allowed');},
    });
    try{
      const recovered=await restarted.callTool('probe_qa',
        {archivedRunId:finished.archivedRunId});
      assert.equal(recovered.status,'ARCHIVED_SOURCE_ONLY');
      assert.equal(recovered.reportedOutcome,'PROBE_REPORTED_PASS');
      assert.deepEqual(recovered.reportedCounts,
        {expected:2,pass:2,fail:0,info:0,look:0});
      assert.equal(recovered.mapTesting,'NOT_TESTED');
      assert.equal(recovered.nativeRunState,'NOT_VERIFIED_AFTER_ARCHIVE_READ');
      const blocked=await restarted.callTool('probe_qa',
        {slot:2,coordinateNative:true});
      assert.equal(blocked.status,'BLOCKED');
      assert.equal(blocked.reason,
        'TESTING_PRIOR_ARCHIVED_RUN_REQUIRES_RECONCILIATION');
      assert.equal(blocked.existingRunId,finished.testingRunId);
      assert.equal(blocked.operatorNextAction,
        'READ_HISTORICAL_RESULT_NO_AUTOMATIC_RESTART');
      assert.equal(await restarted.callTool('probe_qa',{
        archivedRunId:'f'.repeat(32)}).then(x=>x.status),
        'TESTING_ARCHIVE_UNAVAILABLE_OR_INVALID');
    }finally{restarted.shutdown();}
  }finally{
    active.shutdown();
    await rm(root,{recursive:true,force:true});
  }
});

test('Testing history read and archive failure remain fail-closed',async()=>{
  const unconfigured=createHandler(config(),{
    native:async()=>{throw Error('Do not touch native');},
  });
  const missing=await unconfigured.callTool('probe_qa',
    {archivedRunId:'c'.repeat(32)});
  assert.equal(missing.status,'TESTING_ARCHIVE_NOT_CONFIGURED');
  assert.equal(missing.gameplayAcceptance,'NOT_VERIFIED');
  await assert.rejects(()=>unconfigured.callTool('probe_qa',
    {archivedRunId:'invalid'}),/Invalid tool arguments/);
  await assert.rejects(()=>unconfigured.callTool('probe_qa',
    {archivedRunId:'c'.repeat(32),restart:true}),/Invalid tool arguments/);
  assert.throws(()=>parseConfiguration([
    '--testing-archive-dir',path.resolve('Other')],{}),
    /Testing archive requires an absolute Data\/Testing root/);
  const archive=path.join(path.resolve('Data'),'Testing');
  assert.equal(parseConfiguration([
    '--testing-archive-dir',archive],{}).testingArchiveDirectory,archive);
  unconfigured.shutdown();
});

test('coordinated Testing never invents archive saved status on write failure',async()=>{
  const cfg={...config(),testingArchiveDirectory:path.join(
    path.resolve('Data'),'Testing'),
    probeZip:path.resolve('probe.zip'),
    evidenceDirectory:path.resolve('capture')};
  const handler=createHandler(cfg,{
    coordinateTesting:async(_exe,args)=>({
      schema:1,slot:args.slot,runId:args.runId,ownerMatch:true,
      phase:args.operation==='finalize'?'FINAL':'RESERVED',
      reportedStatus:args.operation==='finalize'?args.reportedStatus:null,
      reportDigest:args.operation==='finalize'?args.reportDigest:null,
      provenance:'NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF'}),
    findArchivedTestingSlot:async()=>null,
    archiveTesting:async()=>{throw Error('Private Data folder unavailable');},
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      archiveSha256:'a'.repeat(64),runnerSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64),cache:'UNCHANGED_APPROVED_ZIP'}),
    probeRevisionStatus:async()=> 'UNCHANGED',
    native:async(_exe,operation,slot)=>({
      schema:1,ok:true,operation:'target',slot,pid:402,windowId:802,
      createdFileTime:'9001',sessionId:1,foreground:false,
      visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED'}),
    evidence:async(_root,slot,_n,options)=>options?.probeCheckpoint?
      {slot,status:'CAPTURE_FILE_PRESENT',probeEvents:[
        {kind:'start',arg:'auto',count:1,tick:1,at:null},
        {kind:'result',id:'P01',result:'pass',tick:2,at:null},
        {kind:'done',arg:'auto',count:1,tick:3,at:null}]}:
      {slot,status:'CAPTURE_FILE_PRESENT',
        captureCursor:{kind:'STABLE_CAPTURE_CURSOR_V1'}},
  });
  try{
    await handler.callTool('probe_qa',{slot:2,coordinateNative:true});
    const done=await handler.callTool('probe_qa',
      {slot:2,coordinateNative:true});
    assert.equal(done.nativeRunStatus,'FINAL');
    assert.equal(done.archiveStatus,'ARCHIVE_UNAVAILABLE_OR_CONFLICT');
    assert.equal(done.archivedRunId,undefined);
    assert.equal(done.gameplayAcceptance,'NOT_VERIFIED');
  }finally{handler.shutdown();}
});


test('read-only one-entry Testing inspection does not create or mutate a run',async()=>{
  const calls=[];
  const fake={
    schema:1,slot:2,phase:'RESERVED',runId:'c'.repeat(32),
    ownerTokenMatch:false,reportedStatus:null,reportDigest:null,
    ownerProcessProof:'NOT_BOUND_TO_PROCESS_HANDLE',
    operatorApproval:'NOT_AVAILABLE_IN_PROTOCOL',
    transferAllowed:false,
    recoveryAction:'REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL',
    provenance:'RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF',
  };
  const observer=createHandler(config(),{
    native:async()=>{throw Error('Inspect must never contact Minecraft');},
    coordinateTesting:async()=>{throw Error('Inspection must never reserve');},
    publishActivity:async()=>{throw Error('No activity publishing');},
    probeCatalog:async()=>{throw Error('No source ZIP parsing');},
    evidence:async()=>{throw Error('No content log reading');},
    inspectTesting:async(exe,{slot,ownerId})=>{
      calls.push({exe,slot,ownerId});
      return fake;
    },
  });
  const discovery=(await observer.handle({
    jsonrpc:'2.0',id:2,method:'tools/list'})).result.tools;
  const schema=discovery.find(x=>x.name==='probe_qa').inputSchema;
  assert.equal(schema.properties.inspectRun.type,'boolean');
  const reply=await observer.callTool('probe_qa',{slot:2,inspectRun:true});
  assert.equal(reply.status,'TESTING_RUN_INSPECTION');
  assert.equal(reply.inspection.phase,'RESERVED');
  assert.equal(reply.inspection.transferAllowed,false);
  assert.equal(reply.inspection.ownerProcessProof,'NOT_BOUND_TO_PROCESS_HANDLE');
  assert.equal(reply.archiveReview.status,'NOT_CONFIGURED');
  assert.equal(reply.archiveReview.authorizesTransfer,false);
  assert.equal(reply.operatorNextAction,
    'REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL');
  assert.equal(reply.commandSentByLazyTester,false);
  assert.equal(reply.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(calls.length,1);
  assert.match(calls[0].ownerId,/^[a-f0-9]{32}$/);
  assert.deepEqual(await observer.callTool('probe_qa',{slot:2,inspectRun:true}),reply);
  assert.equal(calls.length,2,'Repeats are independent reads, not new runs');
  await assert.rejects(()=>observer.callTool('probe_qa',{
    slot:2,inspectRun:true,coordinateNative:true}),/Invalid tool arguments/);
  await assert.rejects(()=>observer.callTool('probe_qa',{
    slot:2,inspectRun:false}),/Invalid tool arguments/);
  observer.shutdown();
});

test('Testing native read-only inspections do not infer ownership from absence',async()=>{
  let scans=0;
  const handler=createHandler(config(),{
    inspectTesting:async()=>{scans++;return {
      schema:1,slot:3,phase:'NO_RECORD',runId:null,
      ownerTokenMatch:false,reportedStatus:null,reportDigest:null,
      ownerProcessProof:'NOT_BOUND_TO_PROCESS_HANDLE',
      operatorApproval:'NOT_AVAILABLE_IN_PROTOCOL',
      transferAllowed:false,
      recoveryAction:'CHECK_ARCHIVE_AND_CONTROLLER_EPOCH',
      provenance:'RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF',
    };},
    native:async()=>{throw Error('Read only');},
  });
  const result=await handler.callTool('probe_qa',{slot:3,inspectRun:true});
  assert.equal(scans,1);
  assert.equal(result.inspection.runId,null);
  assert.equal(result.operatorNextAction,'CHECK_ARCHIVE_AND_CONTROLLER_EPOCH');
  assert.equal(result.inspection.transferAllowed,false);
  assert.equal(result.gameplayAcceptance,'NOT_VERIFIED');
  handler.shutdown();
});


test('operator review correlates source-only archive, live parent and conflicts without takeover',async()=>{
  const reserved={
    schema:1,slot:2,phase:'RESERVED',runId:'b'.repeat(32),
    ownerTokenMatch:false,reportedStatus:null,reportDigest:null,
    ownerProcessProof:'BOUND_PARENT_EXIT_CONFIRMED',
    operatorApproval:'NOT_AVAILABLE_IN_PROTOCOL',transferAllowed:false,
    recoveryAction:'REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL',
    provenance:'RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF'};
  let archiveCalls=0,inspectCalls=0;
  let archiveStatus='NO_COMMITTED_SUMMARY_FOUND';
  const state={...config(),testingArchiveDirectory:path.join(
    path.resolve('Data'),'Testing')};
  const handler=createHandler(state,{
    native:async()=>{throw Error('Never read Minecraft or invoke controls');},
    coordinateTesting:async()=>{throw Error('Never reserve Testing');},
    evidence:async()=>{throw Error('Never inspect live game capture');},
    publishActivity:async()=>{throw Error('Never publish');},
    inspectTesting:async()=>{inspectCalls++;return reserved;},
    reviewTestingArchive:async({slot,phase,runId})=>{
      archiveCalls++;
      assert.equal(slot,2);assert.equal(phase,'RESERVED');
      assert.equal(runId,reserved.runId);
      if(archiveStatus==='THROW')throw Error('Private path is unavailable');
      return {schema:1,slot,status:archiveStatus,
        trust:'LOCAL_CHECKSUM_ONLY_NOT_MINECRAFT_OR_OPERATOR_ATTESTATION',
        authorizesTransfer:false};
    },
  });
  try {
    const exited=await handler.callTool('probe_qa',{slot:2,inspectRun:true});
    assert.equal(exited.archiveReview.status,'NO_COMMITTED_SUMMARY_FOUND');
    assert.equal(exited.operatorNextAction,
      'AWAIT_SEPARATE_OPERATOR_APPROVAL_NO_TRANSFER_AVAILABLE');
    assert.equal(exited.inspection.transferAllowed,false);
    assert.equal(exited.archiveReview.authorizesTransfer,false);
    archiveStatus='CONFLICT_RESERVED_HAS_FINAL_ARCHIVE';
    const conflict=await handler.callTool('probe_qa',{slot:2,inspectRun:true});
    assert.equal(conflict.operatorNextAction,
      'REVIEW_TESTING_ARCHIVE_CONFLICT_NO_TRANSFER');
    archiveStatus='THROW';
    const unknown=await handler.callTool('probe_qa',{slot:2,inspectRun:true});
    assert.equal(unknown.archiveReview.status,'INTEGRITY_UNAVAILABLE_OR_CONFLICT');
    assert.equal(unknown.operatorNextAction,
      'REVIEW_TESTING_ARCHIVE_CONFLICT_NO_TRANSFER');
    assert.equal(inspectCalls,3);
    assert.equal(archiveCalls,3);
  }finally{handler.shutdown();}
  assert.equal(testingRecoveryNextAction({
    ...reserved,ownerProcessProof:'BOUND_PARENT_RUNNING_AT_INSPECTION'},
    {status:'NO_COMMITTED_SUMMARY_FOUND'}),
    'WAIT_FOR_LIVE_TESTING_OWNER_NO_TRANSFER');
  assert.equal(testingRecoveryNextAction(reserved,{status:'NOT_CONFIGURED'}),
    'RECONCILE_TESTING_ARCHIVE_BEFORE_ANY_TRANSFER');
  assert.equal(testingRecoveryNextAction({
    ...reserved,phase:'FINAL'},{status:'FINAL_REFERENCE_MATCHES_ARCHIVE'}),
    'READ_FINAL_REPORT_NO_OWNER_TRANSFER');
  assert.equal(testingRecoveryNextAction({
    ...reserved,phase:'NO_RECORD'},{status:'HISTORICAL_SUMMARY_PRESENT'}),
    'READ_HISTORICAL_RESULT_NO_AUTOMATIC_RESTART');
});

test('native Testing inspection admission fails before subprocess on bad input',async()=>{
  await assert.rejects(()=>inspectTestingRunNative('',{
    slot:2,ownerId:'a'.repeat(32)}),/Native Testing controller not configured/);
  await assert.rejects(()=>inspectTestingRunNative(path.resolve('lazy-tester.exe'),{
    slot:0,ownerId:'a'.repeat(32)}),/Invalid read-only Testing inspection/);
  await assert.rejects(()=>inspectTestingRunNative(path.resolve('lazy-tester.exe'),{
    slot:2,ownerId:'UPPERCASEOWNER'}),/Invalid read-only Testing inspection/);
});


test('read-only Testing receipt validation rejects forged takeover/liveness evidence',()=>{
  const noRecord={
    schema:1,slot:2,phase:'NO_RECORD',
    runId:null,ownerTokenMatch:false,reportedStatus:null,reportDigest:null,
    ownerProcessProof:'NOT_BOUND_TO_PROCESS_HANDLE',
    operatorApproval:'NOT_AVAILABLE_IN_PROTOCOL',
    transferAllowed:false,
    recoveryAction:'CHECK_ARCHIVE_AND_CONTROLLER_EPOCH',
    provenance:'RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF',
  };
  const wrap=(inspection)=>({schema:1,ok:true,
    operation:'testing-inspect',inspection});
  assert.deepEqual(parseTestingInspectionReceipt(wrap(noRecord),2),noRecord);
  const reserved={...noRecord,phase:'RESERVED',runId:'b'.repeat(32),
    ownerTokenMatch:false,
    recoveryAction:'REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL'};
  assert.deepEqual(parseTestingInspectionReceipt(wrap(reserved),2),reserved);
  for(const state of ['BOUND_PARENT_RUNNING_AT_INSPECTION',
    'BOUND_PARENT_EXIT_CONFIRMED','BOUND_PARENT_STATUS_UNKNOWN']) {
    const observed={...reserved,ownerProcessProof:state};
    assert.deepEqual(parseTestingInspectionReceipt(wrap(observed),2),observed);
    assert.equal(observed.transferAllowed,false,
      'A pinned parent process is not an operator approval');
    const finished={...observed,phase:'FINAL',reportedStatus:'FAIL',
      reportDigest:'e'.repeat(64),
      recoveryAction:'READ_FINAL_REPORT_NO_OWNER_TRANSFER'};
    assert.deepEqual(parseTestingInspectionReceipt(wrap(finished),2),finished);
    assert.throws(()=>parseTestingInspectionReceipt(wrap({
      ...noRecord,ownerProcessProof:state}),2),
      /TESTING_NATIVE_INSPECTION_RECEIPT_INVALID/);
  }
  const terminal={...reserved,phase:'FINAL',reportedStatus:'FAIL',
    reportDigest:'e'.repeat(64),
    recoveryAction:'READ_FINAL_REPORT_NO_OWNER_TRANSFER'};
  assert.deepEqual(parseTestingInspectionReceipt(wrap(terminal),2),terminal);
  for(const invalid of [
    {...reserved,transferAllowed:true},
    {...reserved,operatorApproval:'APPROVED'},
    {...reserved,ownerProcessProof:'VERIFIED_DEAD'},
    {...reserved,ownerId:'a'.repeat(32)},
    {...reserved,reportedStatus:'PROBE_REPORTED_PASS'},
    {...reserved,recoveryAction:'ALLOW_TAKEOVER'},
    {...reserved,runId:'invalid'},
    {...noRecord,runId:'c'.repeat(32)},
    {...terminal,reportDigest:null},
    {...terminal,phase:'RESERVED'},
    {...terminal,slot:4},
  ]) {
    assert.throws(()=>parseTestingInspectionReceipt(wrap(invalid),2),
      /TESTING_NATIVE_INSPECTION_RECEIPT_INVALID/);
  }
  assert.throws(()=>parseTestingInspectionReceipt({
    ...wrap(noRecord),operation:'testing-reserve'},2),
    /TESTING_NATIVE_INSPECTION_RECEIPT_INVALID/);
});

test('never trusts incomplete or out-of-order native profiles',()=>{
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles.toReversed()},'inspect'));
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles},'launch'));
});

test('stop during pending GPU startup cancels batch without a false success',async()=>{
  const calls=[];
  let enterStart,releaseStart;
  const entered=new Promise(resolve=>{enterStart=resolve;});
  const stalled=new Promise(resolve=>{releaseStart=resolve;});
  const gpuFactory=()=>({
    async start(slot){calls.push(['start',slot]);enterStart();await stalled;return {slot};},
    latest(){throw Error('no frame expected');},
    stop(slots){calls.push(['stop',...slots]);},
    shutdown(){calls.push(['shutdown']);},
  });
  const target=slot=>({schema:1,ok:true,operation:'target',
    slot,pid:700+slot,windowId:4000+slot,
    createdFileTime:'134000000000000001',sessionId:1,
    foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'});
  const handler=createHandler({...config(),
    gpuExecutable:path.resolve('lazy-tester-gpu-capture.exe')},{
    gpuSourceFactory:gpuFactory,
    native:async(_exe,_operation,slot)=>target(slot),
  });
  const pending=handler.callTool('start_visual_observation',
    {slots:[2,3],durationMs:5000});
  await entered;
  const stopped=await handler.callTool('stop_visual_observation',{slots:[2]});
  assert.equal(stopped.status,'STARTUP_CANCELLATION_REQUESTED');
  assert.equal(stopped.startOutcome,'UNCONFIRMED_UNTIL_START_REQUEST_SETTLES');
  releaseStart();
  await assert.rejects(pending,/GPU_OBSERVATION_START_CANCELLED/);
  assert.deepEqual(calls.filter(([op])=>op==='start').map(x=>x[1]),[2]);
  assert.equal((await handler.callTool('read_visual_observation',{slot:2}))
    .metadata.status,'NOT_OBSERVING');
  handler.shutdown();
});

test('log rotation before open refuses evidence from either file',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-evidence-rotation-'));
  const file=path.join(root,'MC02.log');
  const archived=path.join(root,'archived.log');
  try {
    await writeFile(file,
      '2026-10-30T12:00:00.000Z log [ANALYTICS]{"type":"old"}\n');
    const result=await readGameEvidence(root,2,20,{
      openFile:async(filePath,mode)=>{
        await rename(filePath,archived);
        await writeFile(filePath,
          '2026-10-30T12:00:01.000Z log [ANALYTICS]{"type":"new"}\n');
        return open(filePath,mode);
      },
    });
    assert.equal(result.status,'CAPTURE_FILE_UNSTABLE');
    assert.equal(result.reason,'FILE_CHANGED_DURING_READ');
    assert.deepEqual(result.events,[]);
  } finally {await rm(root,{recursive:true,force:true});}
});

test('concurrent append after pinned read cannot produce a mixed evidence snapshot',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-evidence-append-'));
  const file=path.join(root,'MC02.log');
  try {
    await writeFile(file,
      '2026-10-30T12:00:00.000Z log [ANALYTICS]{"type":"before"}\n');
    let appended=false;
    const result=await readGameEvidence(root,2,20,{
      openFile:async(...args)=>{
        const handle=await open(...args);
        return {
          stat:(...options)=>handle.stat(...options),
          read:async(...readArgs)=>{
            const current=await handle.read(...readArgs);
            if(!appended) {
              appended=true;
              await appendFile(file,
                '2026-10-30T12:00:01.000Z log [GAME_RESULTS]{"type":"after"}\n');
            }
            return current;
          },
          close:()=>handle.close(),
        };
      },
    });
    assert.equal(result.status,'CAPTURE_FILE_UNSTABLE');
    assert.deepEqual(result.events,[]);
  } finally {await rm(root,{recursive:true,force:true});}
});

test('short positional reads recover full lines and ignore incomplete tail',async()=>{
  const root=await mkdtemp(path.join(tmpdir(),'lazy-evidence-short-'));
  try {
    await writeFile(path.join(root,'MC02.log'),[
      '2026-10-30T12:00:00.000Z log [ANALYTICS]{"type":"first"}',
      '2026-10-30T12:00:01.000Z log [GAME_RESULTS]{"schema":1,"record_type":"gameplay_result","total_score":2}',
      '2026-10-30T12:00:02.000Z log [ANALYTICS]{"type":"unfinished"',
    ].join('\n'));
    const result=await readGameEvidence(root,2,20,{
      openFile:async(...args)=>{
        const handle=await open(...args);
        return {
          stat:(...options)=>handle.stat(...options),
          read:(buffer,offset,length,position)=>
            handle.read(buffer,offset,Math.min(length,7),position),
          close:()=>handle.close(),
        };
      },
    });
    assert.equal(result.status,'CAPTURE_FILE_PRESENT');
    assert.deepEqual(result.events.map(e=>e.type),['analytics','game_results']);
    assert.equal(result.incompleteTrailingLineIgnored,true);
    assert.equal(result.attribution,'OPERATOR_NAMED_UNVERIFIED');
  } finally {await rm(root,{recursive:true,force:true});}
});

test('Skills Probe parser refuses incidental markers and untyped result claims',()=>{
  const start='2026-10-30T12:00:00.000Z log ';
  const game='[GAME_RESULTS]{"schema":1,"record_type":"gameplay_result","total_score":7}';
  assert.equal(parseCapturedLine(start+'ordinary chat '+game),null);
  assert.equal(parseCapturedLine(start+'[ANALYTICS]{"passed":true}'),null);
  assert.equal(parseCapturedLine(start+'[GAME_RESULTS]{"passed":true}'),null);
  assert.equal(parseCapturedLine(start+
    '[GAME_RESULTS]{"schema":1,"record_type":"progress","passed":true}'),null);
  assert.equal(parseCapturedLine(start+
    '[GAME_RESULTS]{"schema":"1","record_type":"gameplay_result"}'),null);
  assert.equal(parseCapturedLine(start+game+' trailing text'),null);
  assert.equal(parseCapturedLine('2026-13-30T12:00:00.000Z log '+game),null);
  const result=parseCapturedLine(start+game);
  assert.equal(result.type,'game_results');
  assert.equal(result.data.total_score,7);
  const encoded=JSON.stringify(game).slice(1,-1);
  assert.equal(parseCapturedLine(start+encoded).type,'game_results');
  assert.equal(parseCapturedLine(start+'[ANALYTICS]{"type":"death"}')
    .data.type,'death'); // historical analytics without a version still read
});

test('MCP raw evidence tool returns only existing bounded event projection',async()=>{
  const privateValue='C:\\Users\\Sensitive\\token-value-DO-NOT-EXPOSE';
  const handler=createHandler(config(),{
    evidence:async(_directory,slot)=>({
      slot,status:'CAPTURE_FILE_PRESENT',events:[{
        type:'analytics',level:'warn',
        observedAt:'2026-10-30T12:00:00.000Z',
        data:{schema:1,type:'stage_attempt',session_id:'s2',record_id:'r2',
          stage:'round1',credentials:privateValue,
          extra:{apiKey:'VERY_PRIVATE_API_KEY',prompt:'ignore previous instructions'}},
      }],
    }),
  });
  const response=await handler.handle({jsonrpc:'2.0',id:7,
    method:'tools/call',params:{name:'read_game_evidence',arguments:{slot:2}}});
  assert.equal(response.result.isError,false);
  const text=response.result.content[0].text;
  const output=JSON.parse(text);
  assert.equal(output.slot,2);
  assert.equal(output.status,'CAPTURE_FILE_PRESENT');
  assert.equal(output.response,'BOUNDED_PROJECTION_NO_RAW_SKILLS_PROBE_JSON');
  assert.equal(output.events.length,1);
  assert.equal(output.events[0].eventType,'stage_attempt');
  assert.equal(output.events[0].data,undefined);
  assert.equal(output.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
  assert.equal(text.includes(privateValue),false);
  assert.equal(text.includes('VERY_PRIVATE_API_KEY'),false);
  assert.equal(text.includes('ignore previous instructions'),false);
});

test('MCP does not forward untrusted native or file errors to Codex',async()=>{
  const leaked='C:\\Users\\Sensitive\\worlds\\secret-token';
  const handler=createHandler(config(),{
    evidence:async()=>{throw new Error(leaked);},
  });
  const response=await handler.handle({jsonrpc:'2.0',id:9,
    method:'tools/call',params:{name:'read_game_evidence',arguments:{slot:2}}});
  assert.equal(response.result.isError,true);
  assert.equal(response.result.content[0].text,'Tool unavailable; details withheld');
  assert.equal(JSON.stringify(response).includes(leaked),false);
  // Known, explicit application policy errors retain useful stable text.
  const protectedSlot=await handler.handle({jsonrpc:'2.0',id:10,
    method:'tools/call',params:{name:'launch_profile',arguments:{slot:1}}});
  assert.equal(protectedSlot.result.isError,true);
  assert.equal(protectedSlot.result.content[0].text,
    'MC01 is reserved for the human host in hybrid mode');
});

test('MCP bounds serialized text and image output before responding',async()=>{
  const privateValue='PRIVATE_NATIVE_PAYLOAD';
  const huge=privateValue.repeat(10_000);
  // Core text serialization still rejects an oversized object, while
  // native profiles are now projected to a small fixed allowlist before
  // reaching that boundary.
  assert.throws(()=>boundedMcpText({extra:huge}),
    /MCP_TEXT_RESPONSE_TOO_LARGE/);
  const inspect=createHandler(config(),{
    native:async()=>({...snapshot('inspect'),extra:huge}),
  });
  const result=await inspect.handle({jsonrpc:'2.0',id:1,
    method:'tools/call',params:{name:'inspect_profiles',arguments:{}}});
  assert.equal(result.result.isError,false);
  assert.equal(JSON.parse(result.result.content[0].text).profiles.length,5);
  assert.equal(JSON.stringify(result).includes(privateValue),false);

  const target=slot=>({schema:1,ok:true,operation:'target',
    slot,pid:1000+slot,windowId:2000+slot,
    createdFileTime:'134000000000000001',sessionId:1,
    foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
    gameplayInput:'NOT_VERIFIED'});
  const imageHandler=createHandler({...config(),
    cuaExecutable:path.resolve('cua-driver.exe')},{
    native:async(_exe,_operation,slot)=>target(slot),
    capture:async()=>({mimeType:'image/png',
      imageBase64:'A'.repeat(8*1024*1024+1),width:800,height:600,
      sha256:'a'.repeat(64)}),
  });
  const image=await imageHandler.handle({jsonrpc:'2.0',id:2,
    method:'tools/call',params:{name:'observe_client',arguments:{slot:2}}});
  assert.equal(image.result.isError,true);
  assert.equal(image.result.content[0].text,'MCP_IMAGE_RESPONSE_TOO_LARGE');
  assert.equal(image.result.content.some(item=>item.type==='image'),false);
});

test('MCP native profile reports expose only the five allowlisted status fields',async()=>{
  const secret='C:\\Users\\Sensitive\\Education\\worlds\\secret-token';
  const unsafe=operation=>({
    ...snapshot(operation),
    evidence:secret,
    dataStoragePath:secret,
    accountAccessToken:secret,
    profiles:snapshot(operation).profiles.map(p=>({
      ...p,issue:secret,profilePath:secret,
      process:{pid:9042,commandLine:secret},
      microsoftAccount:{identifier:secret},
    })),
  });
  const native=async(_exe,operation)=>unsafe(operation);
  const h=createHandler(config(),{native});
  for(const [name,args,operation] of [
    ['inspect_profiles',{},'inspect'],
    ['launch_profile',{slot:2},'launch'],
  ]) {
    const direct=await h.callTool(name,args);
    assert.equal(direct.operation,operation);
    assert.equal(direct.evidence,
      'NATIVE_SESSION_ONLY_NOT_GAMEPLAY_OR_ACCOUNT_SIGNIN');
    assert.deepEqual(Object.keys(direct.profiles[0]),
      ['slot','state','hasWindow','responding','needsAttention']);
    assert.equal(JSON.stringify(direct).includes(secret),false);
    const wire=await h.handle({jsonrpc:'2.0',id:name,
      method:'tools/call',params:{name,arguments:args}});
    assert.equal(wire.result.isError,false);
    assert.equal(JSON.stringify(wire).includes(secret),false);
  }
  const context=await h.handle({jsonrpc:'2.0',id:3,
    method:'tools/call',params:{name:'get_test_context',
      arguments:{slots:[2]}}});
  assert.equal(context.result.isError,false);
  const parsed=JSON.parse(context.result.content[0].text);
  assert.deepEqual(Object.keys(parsed.nativeProfiles[1]),
    ['slot','state','hasWindow','responding','needsAttention']);
  assert.equal(parsed.peerPlan.hostWorld,'NOT_OBSERVED');
  assert.equal(JSON.stringify(context).includes(secret),false);
});

test('native reports fail closed on malformed required fields after sanitization',async()=>{
  const unsafe={...snapshot('inspect'),profiles:[
    ...snapshot('inspect').profiles.slice(0,4),
    {...snapshot('inspect').profiles[4],
      responding:'yes',privateWorld:'C:\\private-world'},
  ]};
  assert.throws(()=>validateNativeReport(unsafe,'inspect'),
    /Invalid profile state/);
  assert.throws(()=>validateNativeReport(unsafe,'launch'),
    /Native result unavailable/);
  const h=createHandler(config(),{native:async()=>unsafe});
  const wire=await h.handle({jsonrpc:'2.0',id:4,
    method:'tools/call',params:{name:'inspect_profiles',arguments:{}}});
  assert.equal(wire.result.isError,true);
  assert.equal(wire.result.content[0].text,
    'Tool unavailable; details withheld');
  assert.equal(JSON.stringify(wire).includes('private-world'),false);
});

test('MC02 experiment uses one input CLI and delegates authoritative focus checks to native owner',async()=>{
  const optin='I_ACCEPT_EXPERIMENTAL_MC02_INPUT';
  assert.throws(()=>parseConfiguration([
    '--mc02-input-experimental','on'],{}),/explicit native opt-in/);
  assert.throws(()=>parseConfiguration([
    '--mode','five-ai','--mc02-input-experimental','on'],
    {LAZY_TESTER_MC02_VIRTUAL_INPUT:optin}),/human-plus-four-ai/);
  const configOpt=parseConfiguration([
    '--exe',path.resolve('lazy-tester.exe'),
    '--mc02-input-experimental','on'],
    {LAZY_TESTER_MC02_VIRTUAL_INPUT:optin});
  assert.equal(configOpt.experimentalMc02InputEnabled,true);
  const nativeCalls=[];
  const native=async(_exe,operation,slot)=>{
    nativeCalls.push([operation,slot]);
    if(operation==='launch-mc02-input' && slot===2) {
      return snapshot(operation);
    }
    throw Error('No separate target CLI preflight may run during input');
  };
  let writes=0;
  const receipt={schema:1,slot:2,gameplayObserved:false,
    status:'NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED'};
  const enabled=createHandler(configOpt,{
    native,mc02Input:async(_exe,packet)=>{
      writes++;
      const {requestedAtMs,...action}=packet;
      assert.equal(Number.isSafeInteger(requestedAtMs),true);
      assert.ok(requestedAtMs<=Date.now());
      assert.deepEqual(action,{keys:17,dx:10,dy:-5,ttlMs:120});
      return receipt;
    },
  });
  const disabled=createHandler(config());
  assert.equal((await disabled.handle({jsonrpc:'2.0',id:1,method:'tools/list'}))
    .result.tools.some(x=>x.name==='send_experimental_mc02_input'),false);
  assert.equal((await enabled.handle({jsonrpc:'2.0',id:2,method:'tools/list'}))
    .result.tools.some(x=>x.name==='send_experimental_mc02_input'),true);
  // Experimental MC02 launch must create the channel in the retained
  // controller; a plain Launch lacks that authorization.
  assert.equal((await enabled.callTool('launch_profile',{slot:2})).operation,
    'launch-mc02-input');
  const input={slot:2,buttons:['move_forward','jump'],
    lookDX:10,lookDY:-5,ttlMs:120};
  const message={jsonrpc:'2.0',id:3,method:'tools/call',
    params:{name:'send_experimental_mc02_input',arguments:input}};
  const result=await enabled.handle(message);
  assert.equal(result.result.isError,false);
  assert.equal(JSON.parse(result.result.content[0].text).gameplayObserved,false);
  assert.equal(writes,1);
  // The native controller already pins both process/HWND objects and
  // checks MC01 foreground under those pins. No duplicated CLI preflight
  // is permitted inside the 500ms command lifetime.
  assert.deepEqual(nativeCalls,[['launch-mc02-input',2]]);
  let nativeRefusals=0;
  const protectedHandler=createHandler(configOpt,{
    native,
    mc02Input:async()=>{
      nativeRefusals++;
      throw Error('Native request refused: NATIVE_REFUSED');
    },
  });
  const rejected=await protectedHandler.handle(message);
  assert.equal(rejected.result.isError,true);
  assert.equal(rejected.result.content[0].text,
    'Native request refused: NATIVE_REFUSED');
  assert.equal(nativeRefusals,1);
  assert.equal(writes,1);
  assert.deepEqual(nativeCalls,[['launch-mc02-input',2]]);
  await assert.rejects(()=>enabled.callTool('send_experimental_mc02_input',
    {...input,buttons:['move_forward','move_backward']}),/INVALID_COMMAND/);
});

test('MCP five-slot control-frame preview never contacts native controller',async()=>{
  let nativeCalls=0;
  const h=createHandler(config(),{native:async()=>{
    nativeCalls++;
    throw Error('Unexpected native call during source-only planning');
  }});
  const output=await h.callTool('plan_player_actions',{
    includeClientFrames:true,
    actions:[
      {slot:2,kind:'move_forward',atMs:0,durationMs:200},
      {slot:3,kind:'attack',atMs:0,durationMs:100},
      {slot:4,kind:'jump',atMs:50,durationMs:100},
      {slot:5,kind:'use',atMs:50,durationMs:100},
    ],
  });
  assert.equal(nativeCalls,0);
  assert.equal(output.execution,'NOT_DISPATCHED');
  assert.deepEqual(output.clients,[2,3,4,5]);
  assert.deepEqual(output.releaseAllSlots,[2,3,4,5]);
  assert.equal(output.batches[0].frames[0].keyMask,1);
  await assert.rejects(()=>h.callTool('plan_player_actions',{
    includeClientFrames:true,includeTimeline:true,
    actions:[{slot:2,kind:'attack',atMs:0,durationMs:100}],
  }),/Invalid tool arguments/);
  await assert.rejects(()=>h.callTool('plan_player_actions',{
    includeClientFrames:true,
    actions:[{slot:1,kind:'attack',atMs:0,durationMs:100}],
  }),/human-owned/);
});

test('MC02 sequence allows exact-window image capture without unlocking native mutations',async()=>{
  const cfg=parseConfiguration([
    '--exe',path.resolve('lazy-tester.exe'),
    '--cua-exe',path.resolve('cua-driver.exe'),
    '--mc02-input-experimental','on',
  ],{LAZY_TESTER_MC02_VIRTUAL_INPUT:'I_ACCEPT_EXPERIMENTAL_MC02_INPUT'});
  let onFirstWrite,resumeFirstWrite;
  const entered=new Promise(resolve=>{onFirstWrite=resolve;});
  const blocked=new Promise(resolve=>{resumeFirstWrite=resolve;});
  let nativeReads=0,writes=0;
  const target=slot=>({schema:1,ok:true,operation:'target',slot,
    pid:800+slot,windowId:5000+slot,createdFileTime:'134000000000000001',
    sessionId:1,foreground:false,visible:true,minimized:false,
    verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'});
  const handler=createHandler(cfg,{
    native:async(_exe,op,slot)=>{
      assert.equal(op,'target');nativeReads++;return target(slot);
    },
    capture:async()=>({mimeType:'image/png',imageBase64:'iVBORw0KGgo=',
      width:640,height:360,sha256:'b'.repeat(64)}),
    mc02Batch:async(_exe,pulses,{signal})=>{
      writes++;
      assert.ok(pulses[0].keys);
      onFirstWrite();await blocked;
      if(signal.aborted)throw Error('MC02_SEQUENCE_ABORTED');
      return {schema:1,slot:2,
        status:'EXPERIMENTAL_MC02_SEQUENCE_TRANSPORT_ACKNOWLEDGED',
        pulsesAcknowledged:pulses.length,gameplayObserved:false};
    },
    mc02Input:async(_exe,packet)=>{
      writes++;
      assert.equal(packet.keys,0);
      return {schema:1,slot:2,
        status:'NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  });
  const running=handler.callTool('run_experimental_mc02_actions',{
    slot:2,actions:[{slot:2,kind:'move_forward',atMs:0,durationMs:500}],
  });
  await entered;
  const observed=await handler.callTool('observe_client',{slot:2});
  assert.equal(observed.metadata.background,true);
  assert.equal(observed.metadata.gameInput,'BACKGROUND_MINECRAFT_INPUT_UNVERIFIED');
  assert.equal(nativeReads,2);
  await assert.rejects(()=>handler.callTool('observe_client',{slot:1}),/human host/);
  await assert.rejects(()=>handler.callTool('launch_profile',{slot:3}),/in progress/);
  const stop=await handler.callTool('stop_experimental_mc02_actions',{slot:2});
  assert.equal(stop.status,'CANCELLATION_REQUESTED');
  resumeFirstWrite();
  await assert.rejects(running,/MC02_SEQUENCE_ABORTED/);
  assert.equal(writes,2);
  handler.shutdown();
});
test('ordinary pending screenshot blocks mutations and simultaneous screenshots',async()=>{
  let onCapture,resumeCapture;
  const entered=new Promise(resolve=>{onCapture=resolve;});
  const blocked=new Promise(resolve=>{resumeCapture=resolve;});
  const handler=createHandler({...config(),cuaExecutable:path.resolve('cua-driver.exe')},{
    native:async(_exe,op,slot)=>({schema:1,ok:true,operation:'target',
      slot,pid:900+slot,windowId:6000+slot,
      createdFileTime:'134000000000000001',sessionId:1,
      foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED'}),
    capture:async()=>{onCapture();await blocked;return {
      mimeType:'image/png',imageBase64:'iVBORw0KGgo=',width:640,height:360};},
  });
  const reading=handler.callTool('observe_client',{slot:2});
  await entered;
  await assert.rejects(()=>handler.callTool('launch_profile',{slot:3}),/in progress/);
  await assert.rejects(()=>handler.callTool('observe_client',{slot:3}),/in progress/);
  resumeCapture();
  const image=await reading;
  assert.equal(image.metadata.slot,2);
  handler.shutdown();
});

test('urgent MC02 native neutral write may overlap read-only capture without unlocking other mutations',async()=>{
  const cfg=parseConfiguration([
    '--exe',path.resolve('lazy-tester.exe'),
    '--cua-exe',path.resolve('cua-driver.exe'),
    '--mc02-input-experimental','on',
  ],{LAZY_TESTER_MC02_VIRTUAL_INPUT:'I_ACCEPT_EXPERIMENTAL_MC02_INPUT'});
  let captureReached,finishCapture;
  const reached=new Promise(resolve=>{captureReached=resolve;});
  const held=new Promise(resolve=>{finishCapture=resolve;});
  let releases=0;
  const handler=createHandler(cfg,{
    native:async(_exe,op,slot)=>({schema:1,ok:true,operation:'target',
      slot,pid:2000+slot,windowId:2100+slot,
      createdFileTime:'134000000000000001',sessionId:1,
      foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED'}),
    capture:async()=>{captureReached();await held;return {
      mimeType:'image/png',imageBase64:'iVBORw0KGgo=',
      width:640,height:360};},
    mc02Input:async(_exe,packet)=>{
      assert.deepEqual([packet.keys,packet.dx,packet.dy,packet.ttlMs],[0,0,0,1]);
      releases++;return {schema:1,slot:2,
        status:'NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  });
  const reading=handler.callTool('observe_client',{slot:2});
  await reached;
  const stop=await handler.callTool('stop_experimental_mc02_actions',{slot:2});
  assert.equal(stop.status,'MC02_NATIVE_NEUTRAL_STATE_WRITE_ACKNOWLEDGED');
  assert.equal(stop.actualGameKeyRelease,'NOT_VERIFIED');
  assert.equal(releases,1);
  await assert.rejects(()=>handler.callTool('launch_profile',{slot:3}),/in progress/);
  finishCapture();
  const frame=await reading;
  assert.equal(frame.metadata.slot,2);
  handler.shutdown();
});

test('raw experimental MC02 input helper refuses contradictory native key masks before subprocess',async()=>{
  const executable=path.resolve('lazy-tester.exe');
  for(const keys of [3,12,15,255]) {
    await assert.rejects(()=>sendExperimentalMc02Native(executable,{
      keys,dx:0,dy:0,ttlMs:150,requestedAtMs:Date.now(),
    }),/MC02_INPUT_INVALID_COMMAND/);
  }
});

test('five-client native input experiment is explicit, source only and mode-bound',()=>{
  const env={LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT:'I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT'};
  assert.equal(parseConfiguration(['--mode','five-ai',
    '--multi-client-input-experimental','on'],env).experimentalMultiClientInputEnabled,true);
  assert.throws(()=>parseConfiguration(['--multi-client-input-experimental','on'],{}));
  assert.throws(()=>parseConfiguration([
    '--multi-client-input-experimental','on','--mc02-input-experimental','on',
  ],{...env,LAZY_TESTER_MC02_VIRTUAL_INPUT:'I_ACCEPT_EXPERIMENTAL_MC02_INPUT'}));
});

test('four/five opted-in AI action streams preserve per-slot neutral cleanup',async()=>{
  const receipts=[],launches=[];
  const native=async(_exe,op,slot)=>{launches.push([op,slot]);return snapshot(op);};
  const aiInput=async(_exe,frame)=>{
    receipts.push(['release',frame.slot,frame.keys]);
    return {schema:1,slot:frame.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false};
  };
  const aiMultiBatch=async(_exe,batches)=>{
    receipts.push(['batch',...batches.map(b=>b.slot)]);
    return {schema:1,slots:batches.map(b=>b.slot),
      status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
      gameplayObserved:false};
  };
  const make=(slot)=>({slot,kind:'move_forward',atMs:0,durationMs:320});
  const hybrid=createHandler({...config(),experimentalMultiClientInputEnabled:true},
    {native,aiInput,aiMultiBatch,aiLease:leaseAck});
  const initialInput=(await hybrid.callTool('agent_capabilities',{})).activity;
  assert.equal(initialInput.displayHint,'ATTENTION');
  assert.deepEqual(initialInput.items.filter(x=>x.kind==='AI_INPUT')
    .map(x=>x.status),Array(4).fill('STARTUP_RECONCILIATION_REQUIRED'));
  const list=await hybrid.handle({jsonrpc:'2.0',id:21,method:'tools/list'});
  assert.ok(list.result.tools.some(t=>t.name==='run_experimental_multi_client_actions'));
  await assert.rejects(()=>hybrid.callTool('run_experimental_multi_client_actions',
    {actions:[make(1)]}));
  assert.equal(receipts.length,0);
  // New opted-in MCP requires explicit native neutral ACK for any existing
  // games; it must not assume another MCP exited with no keys held.
  await hybrid.callTool('stop_experimental_multi_client_actions',{
    slots:[2,3,4,5],
  });
  assert.equal((await hybrid.callTool('agent_capabilities',{})).activity.items
    .some(x=>x.kind==='AI_INPUT'),false,
    'Confirmed native neutral/release clears the local recovery hold only');
  receipts.length=0;
  const four=await hybrid.callTool('run_experimental_multi_client_actions',
    {actions:[2,3,4,5].map(make)});
  assert.deepEqual(four.slots,[2,3,4,5]);
  assert.deepEqual(receipts.map(r=>r[0]),['batch','release','release','release','release']);
  assert.deepEqual(receipts.slice(1).map(r=>r[1]),[2,3,4,5]);
  await hybrid.callTool('launch_profile',{slot:5});
  assert.deepEqual(launches.at(-1),['launch-ai-hybrid',5]);

  const full=createHandler({...config('five-ai'),experimentalMultiClientInputEnabled:true},
    {native,aiInput,aiMultiBatch,aiLease:leaseAck});
  await full.callTool('stop_experimental_multi_client_actions',{
    slots:[1,2,3,4,5],
  });
  receipts.length=0;
  const five=await full.callTool('run_experimental_multi_client_actions',
    {actions:[1,2,3,4,5].map(make)});
  assert.deepEqual(five.slots,[1,2,3,4,5]);
  assert.deepEqual(receipts.at(-6),['batch',1,2,3,4,5]);
  await full.callTool('launch_profile',{slot:1});
  assert.deepEqual(launches.at(-1),['launch-ai-five',1]);
  const released=await full.callTool('stop_experimental_multi_client_actions',
    {slots:[1,3,5]});
  assert.deepEqual(released.slots,[1,3,5]);
  assert.equal(released.actualGameKeyRelease,'NOT_VERIFIED');
});


// Read-only fixture for an already cached PNG. There is no actual native
// window, GPU worker or screenshot; only the MCP dispatch path is exercised.
function deliveredVisualFixture() {
  const identity={pid:502,windowId:1202,sessionId:1,
    createdFileTime:'134000000000000001'};
  const metadata={schema:1,slot:2,active:true,
    status:'FRAME_BYTES_CHANGED_NOT_SEMANTIC_PROOF',
    freshFrameAvailable:true,sequence:4,successfulSamples:4,
    collectedAt:'2026-11-03T10:00:00.000Z',
    sourceFrameAtMs:Date.parse('2026-11-03T10:00:00.000Z')-100,
    sourceTimeKind:'CUA_QUERY_STARTED',producerSequence:null,
    frameAgeMs:100,captureIdentity:identity,
    width:800,height:600,screenReviewFrameDelivered:false};
  const observed={...metadata};
  const reads=[],stops=[];
  let delivered=false;
  return {identity,observed,reads,stops,wasDelivered:()=>delivered,
    factory:()=>({
      read(slot,{afterSequence=0,includeImage=false}={}){
        assert.equal(slot,2);
        reads.push(includeImage);
        const send=includeImage&&observed.freshFrameAvailable===true&&
          observed.sequence>afterSequence;
        if(send)delivered=true;
        return {metadata:{...observed,hasNewImage:send,
          screenReviewFrameDelivered:delivered},
          imageBase64:send?'a'.repeat(64):null};
      },
      stop(slots){stops.push([...slots]);observed.freshFrameAvailable=false;
        return {status:'STOP_REQUESTED'};},
      start(){throw Error('No screenshot requested');},
      shutdown(){},
    }),
    native:async(_exe,operation,slot)=>({
      schema:1,ok:true,operation:'target',slot,...identity,
      foreground:false,visible:true,minimized:false,
      verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
      gameplayInput:'NOT_VERIFIED',
    }),
  };
}

test('an explicit cached PNG read revalidates the exact live native window once',async()=>{
  const fixture=deliveredVisualFixture();
  let nativeReads=0;
  const handler=createHandler(config(),{
    visualObserverFactory:fixture.factory,
    native:async(...args)=>{nativeReads++;return fixture.native(...args);},
  });
  const metadata=await handler.callTool('read_visual_observation',{
    slot:2,includeImage:false,
  });
  assert.equal(metadata.imageBase64,null);
  assert.equal(nativeReads,0,'metadata-only reads stay local and inexpensive');
  const image=await handler.callTool('read_visual_observation',{
    slot:2,afterSequence:0,includeImage:true,
  });
  assert.equal(image.metadata.hasNewImage,true);
  assert.equal(image.imageBase64,'a'.repeat(64));
  assert.equal(nativeReads,1);
  assert.deepEqual(fixture.reads,[false,false,false,true]);
  const duplicate=await handler.callTool('read_visual_observation',{
    slot:2,afterSequence:4,includeImage:true,
  });
  assert.equal(duplicate.imageBase64,null);
  assert.equal(nativeReads,1,'no new image means no native lookup');
  handler.shutdown();
});

test('a replaced Minecraft process revokes stale PNG before model delivery',async()=>{
  for(const changedField of ['pid','windowId','sessionId','createdFileTime']){
    const fixture=deliveredVisualFixture();
    const handler=createHandler(config(),{
      visualObserverFactory:fixture.factory,
      native:async(...args)=>{
        const v=await fixture.native(...args);
        return {...v,[changedField]:changedField==='createdFileTime'?
          '134000000000000002':v[changedField]+1};
      },
    });
    await assert.rejects(()=>handler.callTool('read_visual_observation',{
      slot:2,includeImage:true,
    }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
    assert.deepEqual(fixture.reads,[false],
      'old PNG must not be marked as delivered before native check');
    assert.deepEqual(fixture.stops,[[2]]);
    handler.shutdown();
  }
});

test('cached screenshot review refuses an unverified replacement, even after prior image delivery',async()=>{
  const fixture=deliveredVisualFixture();
  let replaced=false;
  const handler=createHandler(config(),{
    visualObserverFactory:fixture.factory,
    native:async(...args)=>({
      ...await fixture.native(...args),
      createdFileTime:replaced?'134000000000000002':fixture.identity.createdFileTime,
    }),
  });
  await handler.callTool('read_visual_observation',{slot:2,includeImage:true});
  replaced=true;
  await assert.rejects(()=>handler.callTool('read_visual_observation',{
    slot:2,includeImage:false,
    screenReview:{
      frameSequence:4,sourceFrameAtMs:fixture.observed.sourceFrameAtMs,
      identity:fixture.identity,recognizedLine:'Position: 12, 64, -8',
    },
  }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
  assert.deepEqual(fixture.stops,[[2]]);
  handler.shutdown();
});

test('cache advancement during native identity check cannot leak an unverified PNG',async()=>{
  const fixture=deliveredVisualFixture();
  const handler=createHandler(config(),{
    visualObserverFactory:fixture.factory,
    native:async(...args)=>{
      const v=await fixture.native(...args);
      fixture.observed.sequence++;
      return v;
    },
  });
  await assert.rejects(()=>handler.callTool('read_visual_observation',{
    slot:2,includeImage:true,
  }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
  assert.deepEqual(fixture.reads,[false,false]);
  assert.equal(fixture.wasDelivered(),false,
    'an unverified new cached sequence must never be marked delivered');
  handler.shutdown();
});

function teamVisualFixture(mode='human-plus-four-ai') {
  const slots=mode==='five-ai'?[1,2,3,4,5]:[2,3,4,5];
  const records=new Map(slots.map(slot=>[slot,{
    schema:1,slot,status:'FRAME_BYTES_CHANGED_NOT_SEMANTIC_PROOF',
    active:true,freshFrameAvailable:true,
    sequence:1,successfulSamples:3,frameAgeMs:100,
    collectedAt:'2026-11-01T05:00:00.000Z',
    sourceFrameAtMs:Date.parse('2026-11-01T05:00:00.000Z')-100,
    sourceTimeKind:'CUA_QUERY_STARTED',producerSequence:null,
    captureIdentity:{pid:400+slot,windowId:1400+slot,sessionId:1,
      createdFileTime:'134000000000000001'},
  }]));
  const reads=[];
  let finishObservation=null,captureFailure=null;
  const factory=({onFinish,onCaptureFailure}={})=>{
    finishObservation=onFinish;
    captureFailure=onCaptureFailure;
    return {
    read(slot,{includeImage}={}){
      reads.push([slot,includeImage]);
      return {metadata:{...(records.get(slot)??{
        schema:1,slot,status:'NOT_OBSERVING',active:false,
      })},imageBase64:null};
    },
    start(){throw Error('Unexpected extra capture owner');},
    stop(){return {status:'STOP_REQUESTED'};},
    shutdown(){},
    };
  };
  return {slots,records,reads,factory,
    // Synthetic receipt for unit tests ONLY: a real receipt is created by
    // the native-checked MCP read_visual_observation PNG delivery path.
    markDelivered(selected=slots){
      for(const slot of selected){
        const current=records.get(slot);
        if(!current)throw new Error('Unknown visual fixture slot');
        records.set(slot,{...current,screenReviewFrameDelivered:true});
      }
    },
    native:async(_exe,operation,slot)=>{
      assert.equal(operation,'target');
      assert.ok(slots.includes(slot));
      const identity=records.get(slot)?.captureIdentity;
      assert.ok(identity);
      return {schema:1,ok:true,operation:'target',slot,...identity,
        foreground:false,visible:true,minimized:false,
        verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',
        gameplayInput:'NOT_VERIFIED'};
    },
    revoke(slot,reason='TARGET_REPLACED_REFUSE_CAPTURE'){
      finishObservation?.(slot,reason);
    },
    failCapture(slot,reason='CAPTURE_UNAVAILABLE'){
      captureFailure?.(slot,reason);
    },
  };
}
const aiMove=slot=>({slot,kind:'move_forward',atMs:0,durationMs:320});

test('native visual identity equality includes process birth and Windows session',()=>{
  const identity={pid:403,windowId:1403,createdFileTime:'134000000000000001',sessionId:1};
  assert.equal(sameNativeVisualTarget(identity,identity),true);
  for(const [field,value] of [
    ['pid',404],['windowId',1404],
    ['createdFileTime','134000000000000002'],['sessionId',2],
  ])assert.equal(sameNativeVisualTarget({...identity,[field]:value},identity),false);
  assert.equal(sameNativeVisualTarget(null,identity),false);
});

test('team visual snapshot is metadata-only, names all four AI targets and protects human MC01',async()=>{
  const fixture=teamVisualFixture();
  const h=createHandler(config(),{visualObserverFactory:fixture.factory});
  const state=await h.callTool('get_multi_client_visual_state',{});
  assert.equal(state.status,'RECENT_PER_SLOT_FRAMES_AVAILABLE');
  assert.deepEqual(state.slots,[2,3,4,5]);
  assert.equal(state.imagesIncluded,false);
  assert.equal(state.frames.every(f=>f.sourceTimingUsable===true),true);
  assert.deepEqual(fixture.reads,[[2,false],[3,false],[4,false],[5,false]]);
  assert.equal(state.frames.every(f=>f.imageRead.includeImage===true),true);
  await assert.rejects(()=>h.callTool('get_multi_client_visual_state',
    {slots:[1,2]}),/human host/);
  h.shutdown();
});

test('metadata-only pre-action baseline is refused without native writer or batch I/O',async()=>{
  for(const mode of ['human-plus-four-ai','five-ai']){
    const fixture=teamVisualFixture(mode);
    let acquire=0,batches=0,nativeReads=0;
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      native:async(...args)=>{nativeReads++;return fixture.native(...args);},
      aiLease:async(...args)=>{acquire++;return leaseAck(...args);},
      aiInput:async(_exe,packet)=>({schema:1,slot:packet.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false}),
      aiMultiBatch:async()=>{batches++;throw new Error('DO_NOT_DISPATCH');},
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
    acquire=0;
    const view=await handler.callTool('get_multi_client_visual_state',{});
    const basis=view.frames.map(frame=>frame.baseline);
    assert.equal(view.allSlotsHaveRecentFrames,true);
    assert.equal(view.coordination.observedActionEligible,false);
    await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
      actions:fixture.slots.map(aiMove),basis,
    }),/TEAM_OBSERVATION_IMAGE_NOT_DELIVERED/);
    assert.deepEqual([acquire,batches,nativeReads],[0,0,0],
      'Missing image delivery must refuse BEFORE native target/lease calls');
    fixture.markDelivered(fixture.slots.slice(0,-1));
    await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
      actions:fixture.slots.map(aiMove),basis,
    }),/TEAM_OBSERVATION_IMAGE_NOT_DELIVERED/);
    assert.deepEqual([acquire,batches,nativeReads],[0,0,0]);
    fixture.markDelivered([fixture.slots.at(-1)]);
    const ready=await handler.callTool('get_multi_client_visual_state',{});
    assert.equal(ready.coordination.observedActionEligible,true);
    // A new frame after the image read revokes that frame's delivery proof.
    const changedSlot=fixture.slots[0];
    const prior=fixture.records.get(changedSlot);
    fixture.records.set(changedSlot,{...prior,
      sequence:prior.sequence+1,successfulSamples:prior.successfulSamples+1,
      screenReviewFrameDelivered:false});
    await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
      actions:fixture.slots.map(aiMove),basis:ready.frames.map(frame=>frame.baseline),
    }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
    assert.deepEqual([acquire,batches,nativeReads],[0,0,0]);
    handler.shutdown();
  }
});

test('observed cross-slot action uses exact recent visual basis and one existing native batch',async()=>{
  const fixture=teamVisualFixture();
  const batches=[],released=[];
  const h=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:fixture.native,
    aiLease:leaseAck,
    aiMultiBatch:async(_exe,entries)=>{batches.push(entries);return{
      schema:1,slots:entries.map(e=>e.slot),
      status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
      gameplayObserved:false,
    };},
    aiInput:async(_exe,frame)=>{released.push([frame.slot,frame.keys]);return{
      schema:1,slot:frame.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false,
    };},
  });
  const tools=await h.handle({jsonrpc:'2.0',id:48,method:'tools/list'});
  assert.equal(tools.result.tools.some(t=>t.name==='run_experimental_observed_actions'),true);
  await h.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
  released.length=0;
  const previous=await h.callTool('get_multi_client_visual_state',{});
  const basis=previous.frames.map(f=>f.baseline);
  assert.equal(previous.coordination.observedActionEligible,false,
    'A metadata-only baseline cannot authorize an observed AI action');
  fixture.markDelivered();
  assert.equal((await h.callTool('get_multi_client_visual_state',{}))
    .coordination.observedActionEligible,true);
  const actions=fixture.slots.map(aiMove);
  const receipt=await h.callTool('run_experimental_observed_actions',{actions,basis});
  assert.equal(batches.length,1);
  assert.deepEqual(batches[0].map(b=>b.slot),fixture.slots);
  assert.deepEqual(released,fixture.slots.map(slot=>[slot,0]));
  assert.equal(receipt.actionResult,'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE');
  assert.equal(receipt.gameplayObserved,false);
  assert.match(receipt.actionFingerprint,/^[a-f0-9]{64}$/);
  assert.ok(Date.parse(receipt.transportStartedAt)<=
    Date.parse(receipt.transportCompletedAt));
  assert.deepEqual(receipt.nextObservation.arguments,{slots:fixture.slots,since:basis});
  assert.equal(receipt.decision,'EXTERNAL_AI_DECISION_NOT_VERIFIED_BY_MCP');
  for(const slot of fixture.slots) {
    const old=fixture.records.get(slot);
    fixture.records.set(slot,{...old,successfulSamples:old.successfulSamples+1,
      frameAgeMs:80,collectedAt:'2026-11-01T05:00:01.000Z',
      sourceFrameAtMs:Date.parse('2026-11-01T05:00:01.000Z')-100});
  }
  const after=await h.callTool('get_multi_client_visual_state',
    receipt.nextObservation.arguments);
  assert.equal(after.frames.every(f=>f.sampledSinceBaseline===true),true);
  assert.equal(after.frames.every(f=>f.pngBytesChangedSinceBaseline===false),true);
  assert.equal(after.motionInference,'PNG_BYTES_AND_TIMESTAMPS_NOT_GAMEPLAY_PROOF');
  h.shutdown();
});

test('observed actions reject a replaced live native window before ANY lease or input',async()=>{
  for(const changedField of ['pid','windowId','sessionId','createdFileTime']){
    const fixture=teamVisualFixture();
    let nativeReads=0,leaseMutations=0,inputMutations=0;
    const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      native:async(...arguments_)=>{
        const target=await fixture.native(...arguments_);
        nativeReads++;
        if(target.slot!==3)return target;
        return {...target,[changedField]:changedField==='createdFileTime'?
          '134000000000000999':target[changedField]+1};
      },
      aiLease:async(...args)=>{leaseMutations++;return leaseAck(...args);},
      aiInput:async(_exe,frame)=>({schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false}),
      aiMultiBatch:async()=>{inputMutations++;throw Error('SHOULD_NOT_WRITE_INPUT');},
    });
    // Reconcile the explicit native-input recovery gate before attempting
    // the independently captured visual action. Count only later mutations.
    await handler.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
    leaseMutations=0;
    const view=await handler.callTool('get_multi_client_visual_state',{});
    fixture.markDelivered();
    const actions=fixture.slots.map(aiMove);
    await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
      actions,basis:view.frames.map(f=>f.baseline),
    }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
    assert.deepEqual([leaseMutations,inputMutations],[0,0]);
    assert.deepEqual(nativeReads,[2,3].length);
    handler.shutdown();
  }
});

test('a visual frame changed during async native preflight cannot authorize old AI input',async()=>{
  const fixture=teamVisualFixture();
  let mutated=false,leaseMutations=0;
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:async(...arguments_)=>{
      const live=await fixture.native(...arguments_);
      if(live.slot===3&&!mutated){
        const old=fixture.records.get(3);
        fixture.records.set(3,{...old,sequence:old.sequence+1,
          successfulSamples:old.successfulSamples+1});
        mutated=true;
      }
      return live;
    },
    aiLease:async(...args)=>{leaseMutations++;return leaseAck(...args);},
    aiInput:async(_exe,frame)=>({schema:1,slot:frame.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false}),
    aiMultiBatch:async()=>{throw Error('SHOULD_NOT_DISPATCH');},
  });
  await handler.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
  leaseMutations=0;
  fixture.markDelivered();
  const state=await handler.callTool('get_multi_client_visual_state',{});
  await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
    actions:fixture.slots.map(aiMove),basis:state.frames.map(f=>f.baseline),
  }),/TEAM_OBSERVATION_STALE_OR_REPLACED/);
  assert.equal(mutated,true);
  assert.equal(leaseMutations,0);
  handler.shutdown();
});

test('no opt-in, missing capture, stale image or replaced client cannot enter observed action dispatch',async()=>{
  const fixture=teamVisualFixture();
  let nativeBatches=0;
  const overrides={visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
    aiMultiBatch:async()=>{nativeBatches++;throw Error('UNEXPECTED_NATIVE_MUTATION');}};
  const disabled=createHandler(config(),overrides);
  await assert.rejects(()=>disabled.callTool('run_experimental_observed_actions',
    {actions:fixture.slots.map(aiMove),basis:[]}),/EXPERIMENT_DISABLED/);
  disabled.shutdown();
  const enabled=createHandler({...config(),experimentalMultiClientInputEnabled:true},overrides);
  const state=await enabled.callTool('get_multi_client_visual_state',{});
  const basis=state.frames.map(f=>f.baseline);
  await assert.rejects(()=>enabled.callTool('run_experimental_observed_actions',
    {actions:fixture.slots.map(aiMove),basis:basis.slice(1)}),/STALE_OR_REPLACED/);
  fixture.records.set(3,{...fixture.records.get(3),
    frameAgeMs:5000,freshFrameAvailable:false});
  await assert.rejects(()=>enabled.callTool('run_experimental_observed_actions',
    {actions:fixture.slots.map(aiMove),basis}),/INCOMPLETE/);
  fixture.records.set(3,{...fixture.records.get(3),
    frameAgeMs:100,freshFrameAvailable:true,
    captureIdentity:{...fixture.records.get(3).captureIdentity,windowId:5555}});
  await assert.rejects(()=>enabled.callTool('run_experimental_observed_actions',
    {actions:fixture.slots.map(aiMove),basis}),/STALE_OR_REPLACED/);
  assert.equal(nativeBatches,0);
  enabled.shutdown();
});

test('five-AI visual decision supports MC01 only when explicitly selected as an AI client',async()=>{
  const fixture=teamVisualFixture('five-ai');
  const h=createHandler({...config('five-ai'),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:fixture.native,
    aiLease:leaseAck,
    aiMultiBatch:async(_exe,entries)=>({
      schema:1,slots:entries.map(e=>e.slot),
      status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
      gameplayObserved:false}),
    aiInput:async(_exe,frame)=>({schema:1,slot:frame.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',gameplayObserved:false}),
  });
  await h.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
  const status=await h.callTool('get_multi_client_visual_state',{});
  assert.equal(status.coordination.observedActionEligible,false);
  fixture.markDelivered();
  assert.equal((await h.callTool('get_multi_client_visual_state',{}))
    .coordination.observedActionEligible,true);
  assert.deepEqual(status.slots,[1,2,3,4,5]);
  const result=await h.callTool('run_experimental_observed_actions',{
    actions:fixture.slots.map(aiMove),
    basis:status.frames.map(f=>f.baseline),
  });
  assert.deepEqual(result.slots,[1,2,3,4,5]);
  h.shutdown();
});


test('MCP evaluates four AI action categories only from cached same-process visual metadata',async()=>{
  const fixture=teamVisualFixture();
  let nativeCalls=0;
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
    native:async()=>{nativeCalls++;throw Error('Unexpected native request');},
    aiMultiBatch:async()=>{nativeCalls++;throw Error('Unexpected action dispatch');},
  });
  const advertised=await handler.handle({jsonrpc:'2.0',id:71,method:'tools/list'});
  assert.equal(advertised.result.tools.some(
    t=>t.name==='evaluate_experimental_multi_client_outcomes'),true);
  const before=await handler.callTool('get_multi_client_visual_state',{});
  const receipt={
    schema:1,mode:'human-plus-four-ai',slots:fixture.slots,
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    actionResult:'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE',
    gameplayObserved:false,
    visualBaseline:before.frames.map(f=>f.baseline),
    actionFingerprint:fingerprintActions('human-plus-four-ai',[
      aiMove(2),{slot:3,kind:'look_relative',atMs:0,durationMs:90,deltaX:12,deltaY:0},
      {slot:4,kind:'attack',atMs:0,durationMs:120},aiMove(5),
    ]),
    transportStartedAt:'2026-11-01T05:00:00.250Z',
    transportCompletedAt:'2026-11-01T05:00:01.250Z',
  };
  for(const slot of fixture.slots) {
    const prior=fixture.records.get(slot);
    fixture.records.set(slot,{...prior,sequence:2,successfulSamples:4,
      frameAgeMs:100,collectedAt:'2026-11-01T05:00:02.000Z',
      sourceFrameAtMs:Date.parse('2026-11-01T05:00:02.000Z')-100,
      screenReviewFrameDelivered:slot===2||slot===3});
  }
  const after=await handler.callTool('get_multi_client_visual_state',{
    slots:fixture.slots,since:receipt.visualBaseline});
  assert.equal(after.frames.every(row=>row.sampledSinceBaseline===true),true);
  const result=await handler.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:[
      aiMove(2),{slot:3,kind:'look_relative',atMs:0,durationMs:90,deltaX:12,deltaY:0},
      {slot:4,kind:'attack',atMs:0,durationMs:120},aiMove(5),
    ],
    receipt,after:after.frames.map(f=>f.baseline),
    reviews:[
      {slot:2,category:'movement',assessment:'VISIBLE_CHANGE_REPORTED',
        cue:'The player appears farther from the same wall landmark'},
      {slot:3,category:'camera',assessment:'VISIBLE_CHANGE_REPORTED',
        cue:'A tree shifts toward the left side of the view'},
      {slot:4,category:'attack',assessment:'UNDETERMINED'},
    ],
  });
  assert.equal(result.gameplayAcceptance,'NOT_VERIFIED');
  assert.equal(result.imagesIncluded,false);
  assert.deepEqual(result.slots,[2,3,4,5]);
  assert.equal(result.totals.reportedVisibleChanges,2);
  assert.equal(result.totals.undetermined,2);
  assert.equal(nativeCalls,0);
  assert.equal(fixture.reads.every(([slot,image])=>fixture.slots.includes(slot)&&image===false),true);
  const alteredReceipt={...receipt,actionFingerprint:'f'.repeat(64)};
  await assert.rejects(()=>handler.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:[
      aiMove(2),{slot:3,kind:'look_relative',atMs:0,durationMs:90,deltaX:12,deltaY:0},
      {slot:4,kind:'attack',atMs:0,durationMs:120},aiMove(5),
    ],receipt:alteredReceipt,after:after.frames.map(f=>f.baseline),
  }),/TEAM_ACTION_RECEIPT_MISMATCH/);
  const stale=after.frames.map(f=>f.baseline);
  stale[0]={...stale[0],sequence:999};
  const inconclusive=await handler.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:[
      aiMove(2),{slot:3,kind:'look_relative',atMs:0,durationMs:90,deltaX:12,deltaY:0},
      {slot:4,kind:'attack',atMs:0,durationMs:120},aiMove(5),
    ],receipt,after:stale,
  });
  assert.equal(inconclusive.outcomes[0].postFrameUsable,false);
  assert.equal(inconclusive.outcomes[0].findings[0].status,'NOT_EVALUABLE');
  handler.shutdown();
});

test('MCP model visual change is not credited when post-action PNG was never delivered',async()=>{
  const fixture=teamVisualFixture();
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:async()=>{throw Error('Evaluation must remain metadata-only');},
  });
  const base=await handler.callTool('get_multi_client_visual_state',{});
  const receipt={schema:1,mode:'human-plus-four-ai',slots:fixture.slots,
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    actionResult:'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE',
    gameplayObserved:false,visualBaseline:base.frames.map(frame=>frame.baseline),
    actionFingerprint:fingerprintActions('human-plus-four-ai',fixture.slots.map(aiMove)),
    transportStartedAt:'2026-11-01T05:00:00.250Z',
    transportCompletedAt:'2026-11-01T05:00:01.250Z'};
  for(const slot of fixture.slots){
    const earlier=fixture.records.get(slot);
    fixture.records.set(slot,{...earlier,sequence:2,successfulSamples:4,
      collectedAt:'2026-11-01T05:00:02.000Z',
      sourceFrameAtMs:Date.parse('2026-11-01T05:00:02.000Z')-100});
  }
  const after=await handler.callTool('get_multi_client_visual_state',{
    slots:fixture.slots,since:receipt.visualBaseline});
  const arguments_={actions:fixture.slots.map(aiMove),receipt,
    after:after.frames.map(frame=>frame.baseline),
    reviews:[{slot:3,category:'movement',assessment:'VISIBLE_CHANGE_REPORTED',
      cue:'A claimed moved target appears near the same background wall'}]};
  const rejected=await handler.callTool('evaluate_experimental_multi_client_outcomes',arguments_);
  assert.equal(rejected.outcomes.find(row=>row.slot===3).postFrameUsable,false);
  assert.equal(rejected.outcomes.find(row=>row.slot===3).sourceStatus,
    'POST_IMAGE_NOT_DELIVERED_FOR_REVIEW');
  assert.equal(rejected.totals.reportedVisibleChanges,0);
  const current=fixture.records.get(3);
  fixture.records.set(3,{...current,screenReviewFrameDelivered:true});
  const reviewed=await handler.callTool('evaluate_experimental_multi_client_outcomes',arguments_);
  assert.equal(reviewed.outcomes.find(row=>row.slot===3).postFrameUsable,true);
  assert.equal(reviewed.outcomes.find(row=>row.slot===3).findings[0].status,
    'MODEL_REPORTS_VISUAL_CHANGE_UNVERIFIED');
  assert.equal(reviewed.gameplayAcceptance,'NOT_VERIFIED');
  handler.shutdown();
});

test('MCP feedback evaluation cannot enable multi-client input or evaluate human MC01 in hybrid',async()=>{
  const fixture=teamVisualFixture();
  const disabled=createHandler(config(),{visualObserverFactory:fixture.factory});
  const advertised=await disabled.handle({jsonrpc:'2.0',id:72,method:'tools/list'});
  assert.equal(advertised.result.tools.some(
    x=>x.name==='evaluate_experimental_multi_client_outcomes'),false);
  await assert.rejects(()=>disabled.callTool('evaluate_experimental_multi_client_outcomes',
    {actions:fixture.slots.map(aiMove),receipt:{},after:[]}),/EXPERIMENT_DISABLED/);
  disabled.shutdown();
  const enabled=createHandler({...config(),experimentalMultiClientInputEnabled:true},
    {visualObserverFactory:fixture.factory});
  await assert.rejects(()=>enabled.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:[aiMove(1)],receipt:{},after:[null],
  }),/human-owned/);
  enabled.shutdown();
});

test('five AI outcome feedback covers MC01 but never treats reported view movement as game acceptance',async()=>{
  const fixture=teamVisualFixture('five-ai');
  const handler=createHandler({...config('five-ai'),
    experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
  });
  const first=await handler.callTool('get_multi_client_visual_state',{});
  const receipt={
    schema:1,mode:'five-ai',slots:fixture.slots,
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    actionResult:'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE',
    gameplayObserved:false,visualBaseline:first.frames.map(f=>f.baseline),
    actionFingerprint:fingerprintActions('five-ai',fixture.slots.map(aiMove)),
    transportStartedAt:'2026-11-01T05:00:00.250Z',
    transportCompletedAt:'2026-11-01T05:00:01.250Z',
  };
  for(const slot of fixture.slots) {
    fixture.records.set(slot,{...fixture.records.get(slot),
      sequence:2,successfulSamples:5,frameAgeMs:100,
      collectedAt:'2026-11-01T05:00:02.000Z',
      sourceFrameAtMs:Date.parse('2026-11-01T05:00:02.000Z')-100,
      screenReviewFrameDelivered:slot===1});
  }
  const after=await handler.callTool('get_multi_client_visual_state',{});
  const output=await handler.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:fixture.slots.map(aiMove),receipt,
    after:after.frames.map(f=>f.baseline),reviews:[
      {slot:1,category:'movement',assessment:'VISIBLE_CHANGE_REPORTED',
        cue:'Background bridge appears closer after the movement command'},
    ],
  });
  assert.deepEqual(output.slots,[1,2,3,4,5]);
  assert.equal(output.outcomes[0].findings[0].status,
    'MODEL_REPORTS_VISUAL_CHANGE_UNVERIFIED');
  assert.equal(output.gameplayAcceptance,'NOT_VERIFIED');
  handler.shutdown();
});


test('missing capture-source timestamp prevents experimental observed action dispatch',async()=>{
  const fixture=teamVisualFixture();
  const old=fixture.records.get(3);
  fixture.records.set(3,{...old,sourceFrameAtMs:null,sourceTimeKind:null});
  let dispatches=0;
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
    aiMultiBatch:async()=>{dispatches++;throw Error('Unexpected native input');},
  });
  const team=await handler.callTool('get_multi_client_visual_state',{});
  assert.equal(team.allSlotsHaveRecentFrames,true);
  assert.equal(team.frames[1].sourceTimingUsable,false);
  await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
    actions:fixture.slots.map(aiMove),basis:team.frames.map(f=>f.baseline),
  }),/TEAM_OBSERVATION_TIMING_UNAVAILABLE/);
  assert.equal(dispatches,0);
  handler.shutdown();
});

test('MCP per-client feedback rejects a source timestamp older than action receipt even if delivered later',async()=>{
  const fixture=teamVisualFixture();
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
    native:async()=>{throw Error('Unexpected native IPC');},
    aiMultiBatch:async()=>{throw Error('Unexpected native action');},
  });
  const original=await handler.callTool('get_multi_client_visual_state',{});
  const receipt={
    schema:1,mode:'human-plus-four-ai',slots:fixture.slots,
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    actionResult:'NATIVE_TRANSPORT_ONLY_NOT_GAMEPLAY_EVIDENCE',
    gameplayObserved:false,visualBaseline:original.frames.map(f=>f.baseline),
    actionFingerprint:fingerprintActions('human-plus-four-ai',fixture.slots.map(aiMove)),
    transportStartedAt:'2026-11-01T05:00:00.250Z',
    transportCompletedAt:'2026-11-01T05:00:01.250Z',
  };
  for(const slot of fixture.slots) {
    fixture.records.set(slot,{...fixture.records.get(slot),
      sequence:2,successfulSamples:4,frameAgeMs:60,
      collectedAt:'2026-11-01T05:00:02.000Z',
      // A delayed WGC/preview frame read can arrive AFTER the action
      // even though the native pixel readback was BEFORE completion.
      sourceFrameAtMs:Date.parse('2026-11-01T05:00:00.900Z')});
  }
  const post=await handler.callTool('get_multi_client_visual_state',{
    slots:fixture.slots,since:receipt.visualBaseline});
  assert.equal(post.frames.every(frame=>frame.sampledSinceBaseline===true),true);
  const output=await handler.callTool('evaluate_experimental_multi_client_outcomes',{
    actions:fixture.slots.map(aiMove),receipt,
    after:post.frames.map(frame=>frame.baseline),
    reviews:[{slot:2,category:'movement',assessment:'VISIBLE_CHANGE_REPORTED',
      cue:'The player's position appears changed compared to the wall'}],
  });
  assert.equal(output.outcomes.every(frame=>frame.postFrameUsable===false),true);
  assert.equal(output.outcomes[0].sourceStatus,'FRAME_SOURCE_EVENT_NOT_AFTER_ACTION');
  assert.equal(output.totals.reportedVisibleChanges,0);
  assert.equal(output.gameplayAcceptance,'NOT_VERIFIED');
  handler.shutdown();
});


test('uncertain native batch blocks only touched AI slots until explicit neutral ACK',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],
    ['five-ai',[1,2,3,4,5]],
  ]) {
    const fixture=teamVisualFixture(mode);
    let first=true,failNeutral=0;
    const sends=[],releases=[];
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
      aiMultiBatch:async(_exe,batches)=>{
        sends.push(batches.map(b=>b.slot));
        if(first){first=false;throw new Error('IPC_RESPONSE_UNCERTAIN');}
        return {schema:1,slots:batches.map(b=>b.slot),
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
      aiInput:async(_exe,frame)=>{
        releases.push(frame.slot);
        if(frame.slot===failNeutral)throw new Error('UNCONFIRMED_NEUTRAL');
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots});
    releases.length=0;
    const affected=slots.slice(0,2);
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:affected.map(aiMove),
    }),/IPC_RESPONSE_UNCERTAIN/);
    assert.deepEqual(sends,[affected]);
    assert.deepEqual(releases.slice(0,2),affected);

    const view=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(view.coordination.blockedSlots,affected);
    assert.deepEqual(view.coordination.readySlots,slots.slice(2));
    assert.equal(view.coordination.status,'HOLD_SELECTED_COHORT');
    assert.equal(view.coordination.observedActionEligible,false);
    assert.equal(view.coordination.clients[0].blockReason,
      'UNCERTAIN_NATIVE_INPUT_REQUIRES_EXPLICIT_NEUTRAL');
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(affected[0])],
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    assert.equal(sends.length,1); // NEVER blindly replay the unconfirmed call

    // Healthy slots can be chosen ONLY via a newly explicit action list;
    // the full cohort is never silently reduced by the MCP.
    const healthy=slots.slice(2);
    const ok=await handler.callTool('run_experimental_multi_client_actions',{
      actions:healthy.map(aiMove),
    });
    assert.deepEqual(ok.slots,healthy);
    assert.deepEqual(sends.at(-1),healthy);

    failNeutral=affected[0];
    await assert.rejects(()=>handler.callTool('stop_experimental_multi_client_actions',{
      slots:[affected[0]],
    }),/AI_RELEASE_UNCONFIRMED/);
    const still=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(still.coordination.blockedSlots,affected);
    failNeutral=0;
    const cleared=await handler.callTool('stop_experimental_multi_client_actions',{
      slots:[affected[0]],
    });
    assert.equal(cleared.status,'NATIVE_NEUTRAL_WRITES_ACKNOWLEDGED');
    const partial=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(partial.coordination.blockedSlots,[affected[1]]);
    assert.equal(partial.coordination.clients.find(c=>c.slot===affected[0]).status,
      'VISUAL_SOURCE_READY_ONLY');
    await handler.callTool('stop_experimental_multi_client_actions',{
      slots:[affected[1]],
    });
    const full=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(full.coordination.blockedSlots,[]);
    assert.equal(full.coordination.observedActionEligible,false,
      'Neutral recovery restores source readiness, not an image review');
    fixture.markDelivered();
    assert.equal((await handler.callTool('get_multi_client_visual_state',{slots}))
      .coordination.observedActionEligible,true);
    handler.shutdown();
  }
});

test('revoked visual slot aborts the entire observed native sequence and latches recovery',async()=>{
  const fixture=teamVisualFixture();
  let announce;
  const entered=new Promise(resolve=>{announce=resolve;});
  const released=[];
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:fixture.native,
    aiLease:leaseAck,
    aiMultiBatch:async(_exe,_batches,{signal})=>{
      announce();
      return new Promise((_resolve,reject)=>{
        if(signal.aborted){reject(new Error('AI_SEQUENCE_ABORTED'));return;}
        signal.addEventListener('abort',()=>reject(new Error('AI_SEQUENCE_ABORTED')),
          {once:true});
      });
    },
    aiInput:async(_exe,frame)=>{
      released.push(frame.slot);
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  });
  await handler.callTool('stop_experimental_multi_client_actions',{
    slots:fixture.slots,
  });
  released.length=0;
  const view=await handler.callTool('get_multi_client_visual_state',{});
  fixture.markDelivered();
  const actions=fixture.slots.map(aiMove);
  const pending=handler.callTool('run_experimental_observed_actions',{
    actions,basis:view.frames.map(f=>f.baseline),
  });
  await entered; // async experimental transport is active
  fixture.revoke(3,'TARGET_REPLACED_REFUSE_CAPTURE');
  await assert.rejects(pending,/AI_SEQUENCE_ABORTED/);
  assert.deepEqual(released,fixture.slots);
  const held=await handler.callTool('get_multi_client_visual_state',{});
  assert.deepEqual(held.coordination.blockedSlots,fixture.slots);
  await assert.rejects(()=>handler.callTool('run_experimental_observed_actions',{
    actions,basis:view.frames.map(f=>f.baseline),
  }),/AI_INPUT_RECOVERY_REQUIRED/);
  await handler.callTool('stop_experimental_multi_client_actions',{slots:[3]});
  const stillHeld=await handler.callTool('get_multi_client_visual_state',{});
  assert.deepEqual(stillHeld.coordination.blockedSlots,[2,4,5]);
  assert.equal(stillHeld.coordination.reselect,
    'NEVER_AUTOMATICALLY_DROP_UNREADY_SLOTS');
  handler.shutdown();
});


test('first nonbusy observed capture error requests whole-cohort abort, never partial input replay',async()=>{
  const fixture=teamVisualFixture('five-ai');
  let notifyStart;
  const entered=new Promise(resolve=>{notifyStart=resolve;});
  const released=[];
  const handler=createHandler({...config('five-ai'),
    experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    native:fixture.native,
    aiLease:leaseAck,
    aiMultiBatch:async(_exe,_batches,{signal})=>{
      notifyStart();
      return new Promise((_resolve,reject)=>{
        if(signal.aborted){reject(new Error('AI_SEQUENCE_ABORTED'));return;}
        signal.addEventListener('abort',()=>reject(new Error('AI_SEQUENCE_ABORTED')),
          {once:true});
      });
    },
    aiInput:async(_exe,frame)=>{
      released.push(frame.slot);
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  });
  await handler.callTool('stop_experimental_multi_client_actions',{
    slots:fixture.slots,
  });
  released.length=0;
  const before=await handler.callTool('get_multi_client_visual_state',{});
  fixture.markDelivered();
  const pending=handler.callTool('run_experimental_observed_actions',{
    actions:fixture.slots.map(aiMove),
    basis:before.frames.map(frame=>frame.baseline),
  });
  await entered;
  fixture.failCapture(4,'CAPTURE_UNAVAILABLE'); // no 3-error wait
  await assert.rejects(pending,/AI_SEQUENCE_ABORTED/);
  assert.deepEqual(released,[1,2,3,4,5]);
  const view=await handler.callTool('get_multi_client_visual_state',{});
  assert.deepEqual(view.coordination.blockedSlots,[1,2,3,4,5]);
  assert.equal(view.coordination.observedActionEligible,false);
  handler.shutdown();
});



test('P32 pending lease acquire cannot dispatch after MCP shutdown',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],['five-ai',[1,2,3,4,5]]
  ]) {
    let notify,finish,armed=false,batches=0,neutralWrites=0;
    const entered=new Promise(resolve=>{notify=resolve;});
    const gate=new Promise(resolve=>{finish=resolve;});
    const leases=[];
    const h=createHandler({...config(mode),experimentalMultiClientInputEnabled:true},{
      aiLease:async(_exe,operation,selected,opts)=>{
        leases.push([operation,...selected]);
        if(armed&&operation==='acquire'&&!opts.neutralOnly){
          notify();await gate; // stdin closes before native acquire reply
        }
        return leaseAck(_exe,operation,selected,opts);
      },
      aiInput:async(_exe,frame)=>{
        neutralWrites++;
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
      aiMultiBatch:async()=>{batches++;throw Error('LATE_INPUT_FORBIDDEN');},
    });
    await h.callTool('stop_experimental_multi_client_actions',{slots});
    neutralWrites=0;leases.length=0;armed=true;
    const pending=h.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    });
    await entered;
    h.shutdown();
    h.shutdown(); // idempotent
    finish();
    await assert.rejects(pending,/AI_SEQUENCE_ABORTED/);
    assert.equal(batches,0,'late acquire ACK must not dispatch pulses');
    assert.equal(neutralWrites,0,'undispatched action needs only neutral lease release');
    assert.deepEqual(leases,[
      ['acquire',...slots],...slots.map(slot=>['release',slot])
    ]);
    await assert.rejects(()=>h.callTool('stop_experimental_multi_client_actions',{
      slots,
    }),/MCP_CONNECTION_CLOSED/);
    assert.equal(await h.handle({
      jsonrpc:'2.0',id:99,method:'tools/list',
    }),null,'closed MCP no longer accepts requests');
  }
});

test('P32 observed action preflight stops at disconnect without a new writer',async()=>{
  for(const mode of ['human-plus-four-ai','five-ai']){
    const fixture=teamVisualFixture(mode);
    let notify,finish,armed=false,leases=0,batches=0;
    const entered=new Promise(resolve=>{notify=resolve;});
    const gate=new Promise(resolve=>{finish=resolve;});
    const h=createHandler({...config(mode),experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      native:async(...args)=>{
        if(armed){notify();await gate;armed=false;}
        return fixture.native(...args);
      },
      aiLease:async(...args)=>{leases++;return leaseAck(...args);},
      aiInput:async(_exe,frame)=>({schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false}),
      aiMultiBatch:async()=>{batches++;throw Error('LATE_NATIVE_INPUT');},
    });
    await h.callTool('stop_experimental_multi_client_actions',{slots:fixture.slots});
    leases=0;
    const basis=(await h.callTool('get_multi_client_visual_state',{}))
      .frames.map(frame=>frame.baseline);
    fixture.markDelivered();
    armed=true;
    const pending=h.callTool('run_experimental_observed_actions',{
      actions:fixture.slots.map(aiMove),basis,
    });
    await entered;
    h.shutdown();
    finish();
    await assert.rejects(pending,/MCP_CONNECTION_CLOSED/);
    assert.equal(leases,0,'no lease acquisition after disconnected preflight');
    assert.equal(batches,0,'no action input after stream closure');
  }
});

test('P32 active batch cancellation retains exact-owner neutral cleanup',async()=>{
  const slots=[2,3,4,5],neutral=[],calls=[];
  let signalStart;
  const entered=new Promise(resolve=>{signalStart=resolve;});
  const h=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    aiLease:async(_exe,op,selected,opts)=>{
      calls.push([op,...selected]);
      return leaseAck(_exe,op,selected,opts);
    },
    aiInput:async(_exe,frame)=>{
      neutral.push(frame.slot);
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
    aiMultiBatch:async(_exe,_batch,{signal})=>{
      signalStart();
      return new Promise((_resolve,reject)=>{
        if(signal.aborted){reject(Error('AI_SEQUENCE_ABORTED'));return;}
        signal.addEventListener('abort',()=>reject(Error('AI_SEQUENCE_ABORTED')),
          {once:true});
      });
    },
  });
  await h.callTool('stop_experimental_multi_client_actions',{slots});
  neutral.length=0;calls.length=0;
  const pending=h.callTool('run_experimental_multi_client_actions',{
    actions:slots.map(aiMove),
  });
  await entered;
  h.shutdown();
  await assert.rejects(pending,/AI_SEQUENCE_ABORTED/);
  assert.deepEqual(neutral,slots,
    'in-flight input must still attempt independent exact-owner neutral');
  assert.deepEqual(calls,[
    ['acquire',...slots],...slots.map(slot=>['release',slot])
  ]);
});

test('new 4/5 AI MCP connection does not trust any old channel without explicit reconciliation',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],
    ['five-ai',[1,2,3,4,5]],
  ]){
    const fixture=teamVisualFixture(mode);
    const dispatched=[],neutral=[];
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
      aiMultiBatch:async(_exe,batches)=>{
        dispatched.push(batches.map(batch=>batch.slot));
        return {schema:1,slots:batches.map(batch=>batch.slot),
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
      aiInput:async(_exe,frame)=>{
        neutral.push(frame.slot);
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
    });
    // Creating the MCP never emits unsolicited native game I/O.
    const before=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(before.coordination.blockedSlots,slots);
    assert.deepEqual(before.coordination.pendingReconnectSlots,slots);
    assert.equal(before.coordination.observedActionEligible,false);
    assert.equal(before.coordination.clients[0].blockReason,
      'NEW_MCP_CONNECTION_REQUIRES_NEUTRAL_OR_FRESH_AI_LAUNCH');
    assert.deepEqual(neutral,[]);
    assert.deepEqual(dispatched,[]);
    const capabilities=await handler.callTool('agent_capabilities',{});
    assert.equal(capabilities.independentInput.startupReconciliation,
      'EXPLICIT_NEUTRAL_OR_AUTHENTICATED_NEW_AI_LAUNCH');
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    assert.deepEqual(dispatched,[]);
    // One explicitly requested neutral ACK is sufficient only for its slot.
    await handler.callTool('stop_experimental_multi_client_actions',{
      slots:[slots[0]],
    });
    const partial=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(partial.coordination.readySlots,[slots[0]]);
    assert.deepEqual(partial.coordination.pendingReconnectSlots,slots.slice(1));
    const accepted=await handler.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(slots[0])],
    });
    assert.deepEqual(accepted.slots,[slots[0]]);
    assert.deepEqual(dispatched,[[slots[0]]]);
    // Reusing the remaining previously-running slots is still blocked.
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(slots[1])],
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    await handler.callTool('stop_experimental_multi_client_actions',{
      slots:slots.slice(1),
    });
    const resolved=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(resolved.coordination.pendingReconnectSlots,[]);
    assert.equal(resolved.coordination.observedActionEligible,false,
      'Native neutral acknowledgement does not deliver any visual PNG');
    fixture.markDelivered();
    assert.equal((await handler.callTool('get_multi_client_visual_state',{}))
      .coordination.observedActionEligible,true);
    assert.equal(resolved.coordination.gameplayVerification,'NOT_VERIFIED');
    handler.shutdown();
  }
});

test('fresh authenticated opted-in AI launch clears only that original slot reconnect hold',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],
    ['five-ai',[1,2,3,4,5]],
  ]){
    const fixture=teamVisualFixture(mode);
    const launchCalls=[];
    let failed=false,staleReport=false;
    const native=async(_exe,operation,slot)=>{
      launchCalls.push([operation,slot]);
      if(failed)throw new Error('Native controller unavailable or operation refused');
      const report=snapshot(operation);
      if(operation.startsWith('launch-ai-')){
        const profile=report.profiles[slot-1];
        report.profiles[slot-1]={...profile,
          state:staleReport?'STOPPED':'STARTING',
          needsAttention:false};
      }
      return report;
    };
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
      native,
    });
    const selected=slots[0];
    failed=true;
    await assert.rejects(()=>handler.callTool('launch_profile',{slot:selected}),
      /unavailable or operation refused/);
    failed=false;
    const held=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(held.coordination.pendingReconnectSlots,slots);
    staleReport=true;
    const stale=await handler.callTool('launch_profile',{slot:selected});
    assert.equal(stale.profiles[selected-1].state,'STOPPED');
    assert.deepEqual((await handler.callTool('get_multi_client_visual_state',{}))
      .coordination.pendingReconnectSlots,slots);
    staleReport=false;
    const started=await handler.callTool('launch_profile',{slot:selected});
    assert.equal(started.profiles[selected-1].state,'STARTING');
    const now=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(now.coordination.pendingReconnectSlots,slots.slice(1));
    assert.deepEqual(now.coordination.readySlots,[selected]);
    assert.deepEqual(launchCalls.map(row=>row[0]),[
      mode==='five-ai'?'launch-ai-five':'launch-ai-hybrid',
      mode==='five-ai'?'launch-ai-five':'launch-ai-hybrid',
      mode==='five-ai'?'launch-ai-five':'launch-ai-hybrid',
    ]);
    handler.shutdown();
  }
});

test('restarted MCP conservatively forgets earlier release and requires a new exact-owner neutral ACK',async()=>{
  const fixture=teamVisualFixture();
  const calls=[];
  const deps={
    visualObserverFactory:fixture.factory,
    aiLease:leaseAck,
    aiInput:async(_exe,frame)=>{
      calls.push(frame.slot);
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  };
  const opt={...config(),experimentalMultiClientInputEnabled:true};
  const first=createHandler(opt,deps);
  await first.callTool('stop_experimental_multi_client_actions',{slots:[2]});
  assert.deepEqual((await first.callTool('get_multi_client_visual_state',{}))
    .coordination.pendingReconnectSlots,[3,4,5]);
  first.shutdown();
  const newConnection=createHandler(opt,deps);
  assert.deepEqual((await newConnection.callTool('get_multi_client_visual_state',{}))
    .coordination.pendingReconnectSlots,[2,3,4,5]);
  await assert.rejects(()=>newConnection.callTool('run_experimental_multi_client_actions',{
    actions:[aiMove(2)],
  }),/AI_INPUT_RECOVERY_REQUIRED/);
  await newConnection.callTool('stop_experimental_multi_client_actions',{
    slots:[2],
  });
  assert.deepEqual((await newConnection.callTool('get_multi_client_visual_state',{}))
    .coordination.pendingReconnectSlots,[3,4,5]);
  assert.deepEqual(calls,[2,2]);
  newConnection.shutdown();
});


test('failed neutral on one AI releases all independently ACKed peer leases',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],
    ['five-ai',[1,2,3,4,5]],
  ]){
    const fixture=teamVisualFixture(mode);
    let failing=slots[1],attempts=[];
    const leaseCalls=[];
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      aiLease:async(exe,operation,selected,opts)=>{
        leaseCalls.push([operation,...selected]);
        return leaseAck(exe,operation,selected,opts);
      },
      aiInput:async(_exe,frame)=>{
        attempts.push(frame.slot);
        if(frame.slot===failing)throw Error('NATIVE_NEUTRAL_UNCONFIRMED');
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
    });
    await assert.rejects(()=>handler.callTool('stop_experimental_multi_client_actions',{
      slots,
    }),/AI_RELEASE_UNCONFIRMED/);
    assert.deepEqual(attempts,slots); // all attempted, even after MC03 error
    const healthy=slots.filter(slot=>slot!==failing);
    assert.deepEqual(leaseCalls,[
      ['acquire',...slots],...healthy.map(slot=>['release',slot]),
    ]);
    const state=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(state.coordination.pendingReconnectSlots,[]);
    assert.deepEqual(state.coordination.blockedSlots,[failing]);
    assert.deepEqual(state.coordination.readySlots,healthy);
    assert.equal(state.coordination.clients.find(c=>c.slot===failing).blockReason,
      'UNCERTAIN_NATIVE_INPUT_REQUIRES_EXPLICIT_NEUTRAL');
    assert.equal(state.coordination.gameplayVerification,'NOT_VERIFIED');
    // Only MC03 needs another explicit neutral: previously ACKed peers
    // must not be redundantly released or blocked by the failed client.
    attempts=[];failing=0;
    const recovery=await handler.callTool('stop_experimental_multi_client_actions',{
      slots:[slots[1]],
    });
    assert.deepEqual(recovery.slots,[slots[1]]);
    assert.deepEqual(attempts,[slots[1]]);
    assert.deepEqual(leaseCalls.slice(-2),[
      ['acquire',slots[1]],['release',slots[1]],
    ]);
    const now=await handler.callTool('get_multi_client_visual_state',{});
    assert.deepEqual(now.coordination.blockedSlots,[]);
    assert.deepEqual(now.coordination.pendingReconnectSlots,[]);
    handler.shutdown();
  }
});

test('native writer lease adapter refuses malformed owner and slot groups before any CLI execution',async()=>{
  const executable=path.resolve('lazy-tester.exe');
  await assert.rejects(()=>sendExperimentalAiWriterLeaseNative(
    executable,'acquire',[2],{writerId:'NOT-A-WRITER'}),
    /AI_WRITER_LEASE_REQUIRED/);
  for(const slots of [[],[2,2],[3,2],[0],[6],[1,2,3,4,5,6]]){
    await assert.rejects(()=>sendExperimentalAiWriterLeaseNative(
      executable,'acquire',slots,{writerId:'a'.repeat(32)}),
      /AI_WRITER_LEASE_INVALID/);
  }
  await assert.rejects(()=>sendExperimentalAiWriterLeaseNative(
    executable,'force',[2],{writerId:'a'.repeat(32)}),
    /AI_WRITER_LEASE_INVALID/);
  await assert.rejects(()=>sendExperimentalAiWriterLeaseNative(
    executable,'acquire',[2],{writerId:'a'.repeat(32),mode:'incorrect'}),
    /AI_WRITER_LEASE_REQUIRED/);
});

test('two independent opted-in MCPs cannot both own one native AI slot',async()=>{
  const slots=[2,3,4,5];
  const leaseOwners=new Map(),claimedIds=new Set();
  const aiLease=async(_exe,operation,requested,{writerId})=>{
    assert.match(writerId,/^[a-f0-9]{32}$/);
    claimedIds.add(writerId);
    for(const slot of requested){
      if(operation==='acquire' &&
         leaseOwners.has(slot)&&leaseOwners.get(slot)!==writerId){
        throw new Error('Native request refused: NATIVE_WRITER_BUSY');
      }
      if(operation==='release' && leaseOwners.get(slot)!==writerId){
        throw new Error('Native request refused: NATIVE_WRITER_BUSY');
      }
    }
    for(const slot of requested){
      if(operation==='acquire')leaseOwners.set(slot,writerId);
      else leaseOwners.delete(slot);
    }
    return {schema:1,slots:requested,status:operation==='acquire'?
      'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
      gameplayObserved:false};
  };
  const events=[];
  let started,completeA;
  const inBatch=new Promise(resolve=>{started=resolve;});
  const pendingA=new Promise(resolve=>{completeA=resolve;});
  const aiInput=async(_exe,packet)=>{
    events.push(['neutral',packet.slot]);
    return {schema:1,slot:packet.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false};
  };
  const nativeBatch=(_exe,batches,waiting=false)=>({
    schema:1,slots:batches.map(b=>b.slot),
    status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
    gameplayObserved:false,
  });
  const opt={...config(),experimentalMultiClientInputEnabled:true};
  const first=createHandler(opt,{aiLease,aiInput,
    aiMultiBatch:async(_exe,batches)=>{
      started();
      await pendingA;
      return nativeBatch(_exe,batches);
    },
  });
  const visual=teamVisualFixture();
  const second=createHandler(opt,{aiLease,aiInput,
    visualObserverFactory:visual.factory,
    aiMultiBatch:async(exe,batches)=>nativeBatch(exe,batches),
  });
  await first.callTool('stop_experimental_multi_client_actions',{slots});
  await second.callTool('stop_experimental_multi_client_actions',{slots});
  const actions=slots.slice(0,2).map(aiMove);
  const executing=first.callTool('run_experimental_multi_client_actions',{actions});
  await inBatch;
  assert.equal(leaseOwners.size,2);
  await assert.rejects(()=>second.callTool('run_experimental_multi_client_actions',
    {actions}),/NATIVE_WRITER_BUSY/);
  await assert.rejects(()=>second.callTool('stop_experimental_multi_client_actions',
    {slots:slots.slice(0,2)}),/NATIVE_WRITER_BUSY/);
  const rejected=await second.callTool('get_multi_client_visual_state',{slots});
  assert.deepEqual(rejected.coordination.blockedSlots,[]);
  assert.deepEqual(rejected.coordination.pendingReconnectSlots,[]);
  // Native says definitely denied: the second writer did NOT mutate
  // the original game, so it must not gain a false recovery quarantine.
  assert.equal(leaseOwners.size,2); // no foreign neutral or claim
  completeA();
  const succeeded=await executing;
  assert.deepEqual(succeeded.slots,[2,3]);
  assert.equal(leaseOwners.size,0); // returned after native neutral ACK
  await second.callTool('stop_experimental_multi_client_actions',{
    slots:slots.slice(0,2),
  });
  const after=await second.callTool('run_experimental_multi_client_actions',{actions});
  assert.deepEqual(after.slots,[2,3]);
  assert.equal(leaseOwners.size,0);
  assert.equal(claimedIds.size,2);
  first.shutdown();second.shutdown();
});

test('unconfirmed writer lease release keeps AI input held until explicit recovery',async()=>{
  const slots=[2,3,4,5],fixture=teamVisualFixture();
  let failRelease=true,dispatched=0;
  const calls=[];
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:async(_exe,operation,target,{writerId})=>{
      assert.match(writerId,/^[a-f0-9]{32}$/);
      calls.push([operation,...target]);
      if(operation==='release'&&failRelease){
        throw new Error('Native request outcome UNCERTAIN; do not retry blindly');
      }
      return {schema:1,slots:target,status:operation==='acquire'?
        'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
        gameplayObserved:false};
    },
    aiMultiBatch:async(_exe,batches)=>{
      dispatched++;
      return {schema:1,slots:batches.map(x=>x.slot),
        status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
        gameplayObserved:false};
    },
    aiInput:async(_exe,f)=>({schema:1,slot:f.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false}),
  });
  // Initial startup reconciliation also needs a verified lease return.
  failRelease=false;
  await handler.callTool('stop_experimental_multi_client_actions',{slots});
  failRelease=true;
  await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
    actions:slots.map(aiMove),
  }),/AI_WRITER_LEASE_RELEASE_UNCONFIRMED/);
  assert.equal(dispatched,1);
  const held=await handler.callTool('get_multi_client_visual_state',{});
  assert.deepEqual(held.coordination.blockedSlots,slots);
  await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
    actions:[aiMove(2)],
  }),/AI_INPUT_RECOVERY_REQUIRED/);
  failRelease=false;
  const neutral=await handler.callTool('stop_experimental_multi_client_actions',{slots});
  assert.equal(neutral.writerLease,'NATIVE_RELEASED_AFTER_NEUTRAL_ACK');
  assert.deepEqual((await handler.callTool('get_multi_client_visual_state',{}))
    .coordination.blockedSlots,[]);
  assert.equal(dispatched,1); // no replay of original pulse batch
  assert.equal(calls.some(c=>c[0]==='acquire'&&c.length===5),true);
  handler.shutdown();
});


test('operator stop accepts unsorted slots but native lease receives one canonical ascending cohort',async()=>{
  const calls=[];
  const handler=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    aiLease:async(_exe,operation,slots,{writerId})=>{
      assert.match(writerId,/^[a-f0-9]{32}$/);
      calls.push([operation,...slots]);
      if(operation==='acquire')assert.deepEqual(slots,[2,3,5]);
      else assert.equal(slots.length,1);
      return {schema:1,slots,status:operation==='acquire'?
        'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
        gameplayObserved:false};
    },
    aiInput:async(_exe,frame)=>{
      calls.push(['neutral',frame.slot]);
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
  });
  const stopped=await handler.callTool('stop_experimental_multi_client_actions',{
    slots:[5,2,3],
  });
  assert.deepEqual(stopped.slots,[2,3,5]);
  assert.deepEqual(calls,[
    ['acquire',2,3,5],['neutral',2],['neutral',3],['neutral',5],
    ['release',2],['release',3],['release',5],
  ]);
  handler.shutdown();
});


test('source-only MCP lease request forwards cohort mode and refuses a channel launched for the other mode',async()=>{
  for(const [mode,actual] of [
    ['five-ai','human-plus-four-ai'],
    ['human-plus-four-ai','five-ai'],
  ]) {
    const slots=mode==='five-ai'?[1,2,3,4,5]:[2,3,4,5];
    const visual=teamVisualFixture(mode);
    const claims=[];
    let inputWrites=0;
    const handler=createHandler({
      ...config(mode),experimentalMultiClientInputEnabled:true,
    },{
      visualObserverFactory:visual.factory,
      aiLease:async(_exe,operation,selected,{writerId,mode:requested})=>{
        assert.match(writerId,/^[a-f0-9]{32}$/);
        assert.equal(requested,mode);
        claims.push([operation,...selected]);
        if(operation==='acquire'&&requested!==actual){
          // Same fixed code that the native CLI returns after verifying
          // every original child channel before any lease mutation.
          throw new Error('Native request refused: NATIVE_WRITER_MODE_MISMATCH');
        }
        return {schema:1,slots:selected,status:operation==='acquire'?
          'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
          gameplayObserved:false};
      },
      aiInput:async()=>{inputWrites++;throw Error('FORBIDDEN_WRONG_MODE_INPUT');},
      aiMultiBatch:async()=>{inputWrites++;throw Error('FORBIDDEN_WRONG_MODE_ACTION');},
    });
    await assert.rejects(()=>handler.callTool('stop_experimental_multi_client_actions',{
      slots,
    }),/NATIVE_WRITER_MODE_MISMATCH/);
    assert.deepEqual(claims,[['acquire',...slots]]);
    assert.equal(inputWrites,0);
    const state=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(state.coordination.pendingReconnectSlots,slots);
    assert.equal(state.coordination.clients[0].blockReason,
      'NEW_MCP_CONNECTION_REQUIRES_NEUTRAL_OR_FRESH_AI_LAUNCH');
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    assert.equal(inputWrites,0);
    handler.shutdown();
  }
});



test('P36 proven no-write stale acquire does not quarantine healthy 4/5-AI peers',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],['five-ai',[1,2,3,4,5]],
  ]){
    const fixture=teamVisualFixture(mode);
    const calls=[],neutral=[],sent=[];
    let refusal='none';
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      aiLease:async(exe,operation,selected,opts)=>{
        calls.push([operation,...selected]);
        if(operation==='acquire'&&!opts.neutralOnly&&refusal!=='none'){
          throw new Error(refusal==='stale'?
            'Native request refused: NATIVE_WRITER_EPOCH_STALE':
            'Native request refused: OUTCOME_UNCERTAIN');
        }
        return leaseAck(exe,operation,selected,opts);
      },
      aiInput:async(_exe,frame)=>{
        neutral.push(frame.slot);
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
      aiMultiBatch:async(_exe,batches)=>{
        const selected=batches.map(batch=>batch.slot);
        sent.push(selected);
        return {schema:1,slots:selected,
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots});
    calls.length=0;neutral.length=0;
    const first=slots.slice(0,2);
    refusal='stale';
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:first.map(aiMove),
    }),/NATIVE_WRITER_EPOCH_STALE/);
    assert.deepEqual(calls,[['acquire',...first]],
      'pre-birth/expired epoch is refused before any native lease mutation');
    assert.deepEqual(neutral,[]);
    assert.deepEqual(sent,[]);
    const afterStale=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(afterStale.coordination.blockedSlots,[]);
    assert.deepEqual(afterStale.coordination.pendingReconnectSlots,[]);
    assert.deepEqual(afterStale.coordination.readySlots,slots);
    // In contrast, the original IPC request may have executed before
    // its result was lost. Only the selected action cohort is uncertain.
    const selected=slots.slice(1,3);
    refusal='ambiguous';
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:selected.map(aiMove),
    }),/OUTCOME_UNCERTAIN/);
    assert.deepEqual(sent,[]);
    assert.deepEqual(neutral,[]);
    const blocked=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(blocked.coordination.blockedSlots,selected);
    const healthy=slots.filter(slot=>!selected.includes(slot));
    assert.deepEqual(blocked.coordination.readySlots,healthy);
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(selected[0])],
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    refusal='none';
    const fresh=await handler.callTool('run_experimental_multi_client_actions',{
      actions:healthy.map(aiMove),
    });
    assert.deepEqual(fresh.slots,healthy,
      'healthy clients proceed only by a new explicitly selected action');
    assert.deepEqual(sent,[healthy]);
    const still=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(still.coordination.blockedSlots,selected,
      'a healthy action never clears the uncertain original cohort');
    handler.shutdown();
  }
});

test('P36 stale recovery acquire cannot clear existing reconnect holds',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],['five-ai',[1,2,3,4,5]]
  ]) {
    const selected=[slots[0]],calls=[];
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      aiLease:async(_exe,operation,target,opts)=>{
        calls.push([operation,...target]);
        assert.equal(opts.neutralOnly,true);
        throw Error('Native request refused: NATIVE_WRITER_EPOCH_STALE');
      },
      aiInput:async()=>{throw Error('NO_NEUTRAL_WITHOUT_LEASE');},
      aiMultiBatch:async()=>{throw Error('NO_GAMEPLAY_WITHOUT_LEASE');},
    });
    await assert.rejects(()=>handler.callTool('stop_experimental_multi_client_actions',{
      slots:selected,
    }),/NATIVE_WRITER_EPOCH_STALE/);
    assert.deepEqual(calls,[['acquire',...selected]]);
    const state=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(state.coordination.blockedSlots,slots);
    assert.deepEqual(state.coordination.pendingReconnectSlots,slots);
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:selected.map(aiMove),
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    handler.shutdown();
  }
});

test('malformed native acquisition ACK is uncertain, never treated as a definitive writer refusal',async()=>{
  const fixture=teamVisualFixture();
  let malformed=true,neutralWrites=0,actionsSent=0;
  const slots=[2,3,4,5];
  const handler=createHandler({...config(),
    experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:async(_exe,operation,selected,{writerId,mode})=>{
      assert.match(writerId,/^[a-f0-9]{32}$/);
      assert.equal(mode,'human-plus-four-ai');
      if(malformed&&operation==='acquire') {
        return {schema:1,slots:selected,status:'UNKNOWN_ACK',gameplayObserved:false};
      }
      return {schema:1,slots:selected,status:operation==='acquire'?
        'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
        gameplayObserved:false};
    },
    aiInput:async(_exe,pulse)=>{
      neutralWrites++;
      return {schema:1,slot:pulse.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
    aiMultiBatch:async()=>{actionsSent++;throw Error('UNEXPECTED_ACTION');},
  });
  await assert.rejects(()=>handler.callTool('stop_experimental_multi_client_actions',{
    slots:[2],
  }),/AI_WRITER_LEASE_UNCONFIRMED/);
  const held=await handler.callTool('get_multi_client_visual_state',{slots});
  assert.deepEqual(held.coordination.blockedSlots,slots);
  assert.deepEqual(held.coordination.pendingReconnectSlots,[3,4,5]);
  assert.equal(held.coordination.clients[0].blockReason,
    'UNCERTAIN_NATIVE_INPUT_REQUIRES_EXPLICIT_NEUTRAL');
  assert.equal(neutralWrites,0);
  assert.equal(actionsSent,0);
  await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
    actions:[aiMove(2)],
  }),/AI_INPUT_RECOVERY_REQUIRED/);
  malformed=false;
  await handler.callTool('stop_experimental_multi_client_actions',{slots:[2]});
  const after=await handler.callTool('get_multi_client_visual_state',{slots});
  assert.deepEqual(after.coordination.blockedSlots,[3,4,5]);
  assert.equal(neutralWrites,1);
  handler.shutdown();
});


test('one stopped AI during completed batch does not strand leases of neutral-ACKed peers',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],
    ['five-ai',[1,2,3,4,5]],
  ]){
    const fixture=teamVisualFixture(mode);
    const leaseCalls=[],neutralWrites=[],actionBatches=[];
    const failedSlot=slots[1];
    let dead=false;
    const handler=createHandler({...config(mode),
      experimentalMultiClientInputEnabled:true},{
      visualObserverFactory:fixture.factory,
      aiLease:async(exe,operation,selected,opts)=>{
        leaseCalls.push([operation,...selected]);
        return leaseAck(exe,operation,selected,opts);
      },
      aiInput:async(_exe,frame)=>{
        neutralWrites.push(frame.slot);
        if(dead&&frame.slot===failedSlot) {
          throw new Error('ORIGINAL_CHILD_EXITED');
        }
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
      aiMultiBatch:async(_exe,batches)=>{
        actionBatches.push(batches.map(batch=>batch.slot));
        return {schema:1,slots:batches.map(batch=>batch.slot),
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots});
    leaseCalls.length=0; neutralWrites.length=0;
    dead=true;
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    }),/AI_RELEASE_UNCONFIRMED/);
    const healthy=slots.filter(slot=>slot!==failedSlot);
    assert.deepEqual(neutralWrites,slots);
    assert.deepEqual(actionBatches,[slots]);
    assert.deepEqual(leaseCalls,[
      ['acquire',...slots],...healthy.map(slot=>['release',slot]),
    ]);
    const status=await handler.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(status.coordination.blockedSlots,[failedSlot]);
    assert.deepEqual(status.coordination.readySlots,healthy);
    await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(failedSlot)],
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    // The healthy peers can proceed ONLY as an explicit NEW action
    // after their independently confirmed lease/neutral cleanup.
    const next=await handler.callTool('run_experimental_multi_client_actions',{
      actions:healthy.map(aiMove),
    });
    assert.deepEqual(next.slots,healthy);
    assert.deepEqual(actionBatches,[slots,healthy]);
    assert.deepEqual(leaseCalls.slice(-1-healthy.length),[
      ['acquire',...healthy],...healthy.map(slot=>['release',slot]),
    ]);
    assert.equal(next.gameplayObserved,false);
    handler.shutdown();
  }
});

test('uncertain action delivery still quarantines whole cohort even if peer leases are released',async()=>{
  const fixture=teamVisualFixture();
  const slots=fixture.slots;
  const leaseCalls=[];
  let dead=false,dispatches=0;
  const handler=createHandler({...config(),
    experimentalMultiClientInputEnabled:true},{
    visualObserverFactory:fixture.factory,
    aiLease:async(exe,operation,selected,opts)=>{
      leaseCalls.push([operation,...selected]);
      return leaseAck(exe,operation,selected,opts);
    },
    aiInput:async(_exe,frame)=>{
      if(dead&&frame.slot===3)throw Error('CHILD_DISAPPEARED');
      return {schema:1,slot:frame.slot,
        status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
        gameplayObserved:false};
    },
    aiMultiBatch:async()=>{
      dispatches++;
      throw new Error('IPC_RESPONSE_UNCERTAIN');
    },
  });
  await handler.callTool('stop_experimental_multi_client_actions',{slots});
  leaseCalls.length=0;dead=true;
  await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
    actions:slots.map(aiMove),
  }),/AI_RELEASE_UNCONFIRMED/);
  assert.deepEqual(leaseCalls,[
    ['acquire',...slots],['release',2],['release',4],['release',5],
  ]);
  const held=await handler.callTool('get_multi_client_visual_state',{});
  assert.deepEqual(held.coordination.blockedSlots,slots);
  assert.equal(dispatches,1);
  await assert.rejects(()=>handler.callTool('run_experimental_multi_client_actions',{
    actions:[aiMove(2)],
  }),/AI_INPUT_RECOVERY_REQUIRED/);
  assert.equal(dispatches,1); // NEVER blindly replay uncertain transport
  handler.shutdown();
});


test('every 4/5-AI input transaction carries one fresh native fencing epoch',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],['five-ai',[1,2,3,4,5]],
  ]){
    const held=new Map(),used=[];
    const handler=createHandler({
      ...config(mode),experimentalMultiClientInputEnabled:true,
    },{
      aiLease:async(_exe,operation,selected,{writerId,mode:requested,
          leaseEpoch,neutralOnly})=>{
        assert.match(writerId,/^[a-f0-9]{32}$/);
        assert.match(leaseEpoch,/^[a-f0-9]{30}e1$/);
        assert.equal(requested,mode);
        if(operation==='acquire'){
          assert.deepEqual(selected,slots);
          assert.equal(held.size,0);
          used.push([leaseEpoch,neutralOnly]);
          selected.forEach(slot=>held.set(slot,{writerId,leaseEpoch,neutralOnly}));
        } else {
          assert.equal(selected.length,1,'release must isolate original slot');
          selected.forEach(slot=>{
            assert.equal(held.get(slot)?.leaseEpoch,leaseEpoch);
            held.delete(slot);
          });
        }
        return {schema:1,slots:selected,status:operation==='acquire'?
          'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
          gameplayObserved:false};
      },
      aiInput:async(_exe,frame,{writerId,leaseEpoch})=>{
        assert.equal(held.get(frame.slot)?.writerId,writerId);
        assert.equal(held.get(frame.slot)?.leaseEpoch,leaseEpoch);
        assert.equal(frame.keys,0);
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
      aiMultiBatch:async(_exe,batches,{writerId,leaseEpoch})=>{
        batches.forEach(batch=>{
          const claim=held.get(batch.slot);
          assert.equal(claim?.writerId,writerId);
          assert.equal(claim?.leaseEpoch,leaseEpoch);
          assert.equal(claim?.neutralOnly,false);
        });
        return {schema:1,slots:batches.map(b=>b.slot),
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots});
    await handler.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    });
    await handler.callTool('stop_experimental_multi_client_actions',{slots});
    assert.equal(used.length,3);
    assert.deepEqual(used.map(v=>v[1]),[true,false,true]);
    assert.equal(new Set(used.map(v=>v[0])).size,3);
    assert.equal(held.size,0);
    handler.shutdown();
  }
});


test('departed client AFTER neutral ACK cannot strand healthy peers on lease release',async()=>{
  for(const [mode,slots] of [
    ['human-plus-four-ai',[2,3,4,5]],['five-ai',[1,2,3,4,5]],
  ]) {
    const failed=slots[1],claims=new Map(),calls=[];
    let departed=false,neutralCount=0,actions=0;
    const h=createHandler({...config(mode),experimentalMultiClientInputEnabled:true},{
      aiLease:async(_exe,op,selected,{writerId,leaseEpoch})=>{
        calls.push([op,...selected]);
        if(op==='acquire') {
          for(const slot of selected) {
            assert.equal(claims.has(slot),false);
            claims.set(slot,{writerId,leaseEpoch});
          }
        }else {
          assert.equal(selected.length,1,'one original owner per release');
          const slot=selected[0];
          if(departed&&slot===failed)throw Error('ORIGINAL_CHILD_GONE_AFTER_NEUTRAL');
          assert.equal(claims.get(slot)?.leaseEpoch,leaseEpoch);
          claims.delete(slot);
        }
        return {schema:1,slots:selected,status:op==='acquire'?
          'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
          gameplayObserved:false};
      },
      aiInput:async(_exe,frame)=>{
        neutralCount++;
        return {schema:1,slot:frame.slot,
          status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
          gameplayObserved:false};
      },
      aiMultiBatch:async(_exe,batches)=>{
        actions++;
        return {schema:1,slots:batches.map(x=>x.slot),
          status:'EXPERIMENTAL_MULTI_CLIENT_TRANSPORT_ACKNOWLEDGED',
          gameplayObserved:false};
      },
    });
    await h.callTool('stop_experimental_multi_client_actions',{slots});
    assert.equal(claims.size,0);
    neutralCount=0;calls.length=0;departed=true;
    await assert.rejects(()=>h.callTool('run_experimental_multi_client_actions',{
      actions:slots.map(aiMove),
    }),/AI_WRITER_LEASE_RELEASE_UNCONFIRMED/);
    assert.equal(actions,1);
    assert.equal(neutralCount,slots.length);
    assert.deepEqual(calls,[
      ['acquire',...slots],...slots.map(slot=>['release',slot]),
    ],'no slot is skipped/retried when one child exits before release');
    assert.deepEqual([...claims.keys()],[failed]);
    const state=await h.callTool('get_multi_client_visual_state',{slots});
    assert.deepEqual(state.coordination.blockedSlots,[failed]);
    assert.deepEqual(state.coordination.readySlots,
      slots.filter(slot=>slot!==failed));
    await assert.rejects(()=>h.callTool('run_experimental_multi_client_actions',{
      actions:[aiMove(failed)],
    }),/AI_INPUT_RECOVERY_REQUIRED/);
    assert.equal(actions,1,'uncertain client is never replayed');
    const healthy=slots.filter(slot=>slot!==failed);
    await h.callTool('run_experimental_multi_client_actions',{
      actions:healthy.map(aiMove),
    });
    assert.equal(actions,2,'healthy peers require an explicit new action');
    assert.deepEqual([...claims.keys()],[failed],
      'other clients cannot implicitly rehabilitate the departed slot');
    h.shutdown();
  }
});

test('operator stop isolates a native release failure after all neutral ACKs',async()=>{
  const slots=[2,3,4,5],attempts=[],released=[];
  let failure=false;
  const h=createHandler({...config(),experimentalMultiClientInputEnabled:true},{
    aiLease:async(_exe,op,selected)=>{
      if(op==='release') {
        assert.equal(selected.length,1);
        attempts.push(selected[0]);
        if(failure&&selected[0]===3)throw Error('AMBIGUOUS_RELEASE_RESULT');
        released.push(selected[0]);
      }
      return {schema:1,slots:selected,status:op==='acquire'?
        'AI_NATIVE_WRITER_LEASE_ACQUIRED':'AI_NATIVE_WRITER_LEASE_RELEASED',
        gameplayObserved:false};
    },
    aiInput:async(_exe,frame)=>({schema:1,slot:frame.slot,
      status:'NATIVE_AI_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',
      gameplayObserved:false}),
  });
  await h.callTool('stop_experimental_multi_client_actions',{slots});
  attempts.length=0;released.length=0;failure=true;
  await assert.rejects(()=>h.callTool('stop_experimental_multi_client_actions',{
    slots,
  }),/AI_WRITER_LEASE_RELEASE_UNCONFIRMED/);
  assert.deepEqual(attempts,slots,'one attempted release per original slot');
  assert.deepEqual(released,[2,4,5]);
  const status=await h.callTool('get_multi_client_visual_state',{slots});
  assert.deepEqual(status.coordination.blockedSlots,[3]);
  assert.deepEqual(status.coordination.readySlots,[2,4,5]);
  h.shutdown();
});


test('lease epochs encode a bounded FILETIME creation witness with fresh entropy',()=>{
  const at=2000000000000,nonce=Buffer.alloc(7,0xa5);
  const token=makeAiLeaseEpoch(at,nonce);
  assert.match(token,/^[a-f0-9]{30}e1$/);
  assert.equal(token.length,32);
  assert.equal(BigInt('0x'+token.slice(0,16)),
    (BigInt(at)+11644473600000n)*10000n);
  assert.equal(token.slice(16,30),'a5'.repeat(7));
  assert.equal(makeAiLeaseEpoch(at,Buffer.alloc(7,0x5a)).slice(0,16),
    token.slice(0,16));
  assert.notEqual(makeAiLeaseEpoch(at,Buffer.alloc(7,0x5a)),token);
  assert.notEqual(makeAiLeaseEpoch(at+1,nonce),token);
  for(const invalid of [
    [NaN,nonce],[-1,nonce],[Infinity,nonce],
    [at,Buffer.alloc(6)],[at,'not-entropy']
  ]) {
    assert.throws(()=>makeAiLeaseEpoch(...invalid),/AI_LEASE_EPOCH_REQUIRED/);
  }
});

test('native lease adapter refuses missing/stale fencing epoch before CLI spawn',async()=>{
  const executable=path.resolve('lazy-tester.exe');
  for(const leaseEpoch of [undefined,'WRONG_EPOCH','A'.repeat(32),'1'.repeat(32)]){
    await assert.rejects(()=>sendExperimentalAiWriterLeaseNative(
      executable,'acquire',[2],{
        writerId:'a'.repeat(32),mode:'human-plus-four-ai',
        leaseEpoch,neutralOnly:true,
      }),/AI_LEASE_EPOCH_REQUIRED/);
  }
});

test('delayed old process packet cannot match a newly issued same-writer epoch',()=>{
  // Native AiLeaseBook regression is authoritative for this contract.
  const writerId='a'.repeat(32),old='1'.repeat(32),fresh='2'.repeat(32);
  let native={slot:3,pid:303,birth:101,writerId,epoch:old};
  native={slot:3,pid:404,birth:202,writerId,epoch:fresh};
  const accepted=(epoch,pid,birth)=>native.writerId===writerId &&
    native.epoch===epoch && native.pid===pid && native.birth===birth;
  assert.equal(accepted(old,404,202),false);
  assert.equal(accepted(fresh,303,101),false);
  assert.equal(accepted(fresh,404,202),true);
});

test('team analytics export accepts up to 24 KiB source contract without increasing returned records',()=>{
  const prefix='2026-10-30T12:00:00.000Z log ';
  const teamLine='[ANALYTICS] '+JSON.stringify({
    schema:1,type:'stage_attempt',record_id:'r1',
    extra:'x'.repeat(17*1024),
  });
  assert.ok(Buffer.byteLength(teamLine,'utf8')>16*1024);
  assert.ok(Buffer.byteLength(teamLine,'utf8')<24*1024);
  assert.equal(parseCapturedLine(prefix+teamLine).type,'analytics');
  const oversized='[ANALYTICS] '+JSON.stringify({
    schema:1,type:'stage_attempt',extra:'x'.repeat(25*1024),
  });
  assert.equal(parseCapturedLine(prefix+oversized),null);
});

test('team game-results export_summary is captured but is not a gameplay result',()=>{
  const prefix='2026-10-30T12:00:00.000Z warn ';
  const summary='[GAME_RESULTS] '+JSON.stringify({
    schema:1,record_type:'export_summary',
    counts:{gameplay_result:1,level_score:0},
    failed_counts:{gameplay_result:2},
  });
  const parsed=parseCapturedLine(prefix+summary);
  assert.equal(parsed.type,'game_results');
  assert.equal(parsed.data.record_type,'export_summary');
  const level=parseCapturedLine(prefix+'[GAME_RESULTS] '+
    JSON.stringify({schema:1,record_type:'level_score',total_score:7}));
  assert.equal(level.type,'game_results');
  assert.equal(parseCapturedLine(prefix+'[GAME_RESULTS] '+
    JSON.stringify({schema:1,record_type:'export_summary',
      counts:{level_score:'not a number'}})),null);
  const h=createHandler(config(),{evidence:async(_dir,slot)=>({
    slot,status:'CAPTURE_FILE_PRESENT',events:[parsed,level],
  })});
  return h.callTool('summarize_test_evidence',{slots:[2]}).then(result=>{
    assert.equal(result.resultCount,0);
    assert.equal(result.teamGameResultSummaryCount,1);
    assert.equal(result.teamLevelScoreCount,1);
    assert.equal(result.attentionCount,1);
    assert.equal(result.verdict,'EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE');
  });
});

test('retired [LT_EDU_DIAG] records are not parsed or sent to Codex',()=>{
  const line='2026-10-30T12:00:00.000Z warn '+
    '[LT_EDU_DIAG] {"schema":1,"kind":"summary","session":"lt-old-1"}';
  assert.equal(parseCapturedLine(line),null);
  assert.equal(parseCapturedLine('2026-10-30T12:00:00.000Z warn '+
    '[ANALYTICS] {"schema":1,"type":"death"}').type,'analytics');
  assert.equal(parseCapturedLine('2026-10-30T12:00:00.000Z warn '+
    '[GAME_RESULTS] {"schema":1,"record_type":"gameplay_result"}').type,'game_results');
});

test('two MC slots cannot count one identical local capture inode twice',async()=>{
  let nativeCalls=0;
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      runnerSha256:'a'.repeat(64),archiveSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64)}),
    native:async()=>{nativeCalls++;throw Error('no game operation allowed');},
    evidence:async()=>({status:'CAPTURE_FILE_PRESENT',captureCursor:{
      kind:'STABLE_CAPTURE_CURSOR_V1',fileIdentity:'d'.repeat(64)}})
  });
  const first=await h.callTool('prepare_probe_qa',{slot:2});
  assert.equal(first.status,'PREPARED_SOURCE_ONLY');
  const second=await h.callTool('prepare_probe_qa',{slot:3});
  assert.equal(second.status,'CAPTURE_SLOT_COLLISION');
  assert.equal(second.commandSent,false);
  assert.equal(nativeCalls,0);
  h.shutdown();
});

test('different native log identities remain independent across MC slots',async()=>{
  const h=createHandler({...config(),executable:'',
    probeZip:path.resolve('skills-probe.zip'),
    evidenceDirectory:path.resolve('capture')},{
    probeCatalog:async()=>({autoCount:1,autoIds:['P01'],
      runnerSha256:'a'.repeat(64),archiveSha256:'b'.repeat(64),
      sourceRevision:'c'.repeat(64)}),
    evidence:async(_directory,slot)=>({
      status:'CAPTURE_FILE_PRESENT',captureCursor:{
        kind:'STABLE_CAPTURE_CURSOR_V1',
        fileIdentity:(slot===2?'d':'e').repeat(64)}
    })
  });
  assert.equal((await h.callTool('prepare_probe_qa',{slot:2})).status,'PREPARED_SOURCE_ONLY');
  assert.equal((await h.callTool('prepare_probe_qa',{slot:3})).status,'PREPARED_SOURCE_ONLY');
  h.shutdown();
});
