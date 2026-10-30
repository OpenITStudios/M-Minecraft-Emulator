import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtemp,writeFile,symlink,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {parseConfiguration,parseCapturedLine,readGameEvidence,createHandler,validateNativeReport} from './mcp.mjs';

const snapshot=(operation)=>({schema:1,ok:true,operation,profiles:Array.from({length:5},(_,i)=>({slot:i+1,state:i===0?'RUNNING':'STOPPED',responding:i===0,hasWindow:i===0,needsAttention:false}))});
const config=(mode='human-plus-four-ai')=>({mode,executable:'C:\\App\\lazy-tester.exe',evidenceDirectory:''});

test('modes, absolute paths, no implicit native permissions',()=>{
  assert.equal(parseConfiguration(['--mode','five-ai']).mode,'five-ai');
  assert.equal(parseConfiguration([]).mode,'human-plus-four-ai');
  assert.throws(()=>parseConfiguration(['--mode','random']));
  assert.throws(()=>parseConfiguration(['--evidence-dir','relative']));
  assert.throws(()=>parseConfiguration(['--exe','./lazy-tester.exe']));
});

test('MCP handshake and tool discovery are standard JSON-RPC',async()=>{
  const h=createHandler(config());
  const init=await h.handle({jsonrpc:'2.0',id:1,method:'initialize',params:{protocolVersion:'2025-11-25'}});
  assert.equal(init.result.protocolVersion,'2025-11-25');
  const listing=await h.handle({jsonrpc:'2.0',id:2,method:'tools/list'});
  assert.deepEqual(listing.result.tools.map(x=>x.name),['agent_capabilities','inspect_profiles','launch_profile','read_game_evidence']);
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

test('never trusts incomplete or out-of-order native profiles',()=>{
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles.toReversed()},'inspect'));
  assert.throws(()=>validateNativeReport({schema:1,ok:true,operation:'inspect',profiles:snapshot('inspect').profiles},'launch'));
});
