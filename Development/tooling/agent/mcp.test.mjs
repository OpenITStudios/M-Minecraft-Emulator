import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,symlink,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {parseConfiguration,parseCapturedLine,readGameEvidence,createHandler,validateNativeReport,validateNativeWindowTarget} from './mcp.mjs';

const snapshot=(operation)=>({schema:1,ok:true,operation,profiles:Array.from({length:5},(_,i)=>({slot:i+1,state:i===0?'RUNNING':'STOPPED',responding:i===0,hasWindow:i===0,needsAttention:false}))});
const config=(mode='human-plus-four-ai')=>({mode,executable:'C:\\App\\lazy-tester.exe',evidenceDirectory:''});

test('modes, absolute paths, no implicit native permissions',()=>{
  assert.equal(parseConfiguration(['--mode','five-ai']).mode,'five-ai');
  assert.equal(parseConfiguration([]).mode,'human-plus-four-ai');
  assert.throws(()=>parseConfiguration(['--mode','random']));
  assert.throws(()=>parseConfiguration(['--evidence-dir','relative']));
  assert.throws(()=>parseConfiguration(['--exe','./lazy-tester.exe']));
  assert.throws(()=>parseConfiguration(['--cua-exe','./cua-driver.exe']));
  assert.throws(()=>parseConfiguration(['--gpu-capture-exe','gpu.exe']));
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
  assert.deepEqual(listing.result.tools.map(x=>x.name),['start_visual_observation','read_visual_observation','stop_visual_observation','observe_client','get_computer_use_target','revalidate_computer_use_target','get_test_context','agent_capabilities','inspect_profiles','launch_profile','plan_peer_session','plan_player_actions','summarize_test_evidence','read_game_evidence']);
  assert.equal(await h.handle({jsonrpc:'2.0',method:'notifications/initialized'}),null);
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
  assert.equal(JSON.parse(response.result.content[0].text).background,true);
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
  const stopped=await handler.callTool('stop_visual_observation',{slots:[2]});
  assert.equal(stopped.captureFramesRetained,false);
  handler.shutdown();
});

test('opt-in persistent GPU observation reuses MCP tools and protects MC01',async()=>{
  const calls=[];
  const latestFrame={provider:'NATIVE_WGC_PERSISTENT',
    mimeType:'image/png',imageBase64:'a'.repeat(64),width:960,height:540,
    sha256:'a'.repeat(64),frameFreshness:'CAPTURE_TIMESTAMP_RECENT_GAMEPLAY_NOT_VERIFIED'};
  const gpuFactory=()=>({
    async start(slot){calls.push(['start',slot]);return {slot};},
    latest(slot){calls.push(['latest',slot]);return latestFrame;},
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
        type:'game_results',data:{record_type:'gameplay_result',total_score:8,
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
      '2026-10-30T12:00:02.000Z log [GAME_RESULTS]{"type":"result","passed":true}',
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

test('experimental background input is NOT exposed as an agent tool',async()=>{
  const h=createHandler(config());
  const tools=(await h.handle({jsonrpc:'2.0',id:1,method:'tools/list'})).result.tools;
  assert.equal(tools.some(t=>t.name.includes('input')||t.name.includes('probe')),false);
  assert.equal((await h.handle({jsonrpc:'2.0',id:2,method:'tools/call',
    params:{name:'probe-background',arguments:{slot:2}}})).result.isError,true);
  assert.equal((await h.callTool('agent_capabilities',{}))
    .notAvailable.includes('parallel_independent_keyboard_mouse'),true);
});

test('never trusts incomplete or out-of-order native profiles',()=>{
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles.toReversed()},'inspect'));
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles},'launch'));
});
