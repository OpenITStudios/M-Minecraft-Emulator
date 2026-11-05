// Authored for later validation. Operator has NOT authorized local execution.
import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {inspectPng,validateCuaExecutable,buildCuaWindowQuery,MAX_PNG_BYTES,readBoundedPng,
  buildCuaSemanticWindowQuery,projectCuaSemanticWindow,
  inspectCuaWindowElements} from './visual-capture.mjs';

const png1x1=Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+j6YQAAAAASUVORK5CYII=',
  'base64');

test('Cua executable requires a precise absolute local binary path',()=>{
  assert.equal(validateCuaExecutable(path.resolve('bin','cua-driver.exe')),
    path.resolve('bin','cua-driver.exe'));
  for(const invalid of ['cua-driver.exe','./cua-driver.exe',
    path.resolve('bin','powershell.exe')]) {
    assert.throws(()=>validateCuaExecutable(invalid));
  }
});

test('Cua preview targets one exact window without modifying action snapshots',()=>{
  const query=buildCuaWindowQuery({pid:42,windowId:1234});
  assert.equal(query.pid,42);
  assert.equal(query.window_id,1234);
  assert.equal(query.display_only,true);
  assert.equal(query.include_accessibility_tree,false);
  assert.equal(query.include_screenshot,true);
  assert.equal(query.scope,undefined);
  assert.throws(()=>buildCuaWindowQuery({pid:0,windowId:1234}));
  assert.throws(()=>buildCuaWindowQuery({pid:42,windowId:0}));
});

test('Cua semantic UIA query does not capture pixels or accept a caller HWND override',()=>{
  const target={pid:42,windowId:1234};
  const q=buildCuaSemanticWindowQuery(target);
  assert.equal(q.pid,42);
  assert.equal(q.window_id,1234);
  assert.equal(q.include_screenshot,false);
  assert.equal(q.include_accessibility_tree,true);
  assert.equal(q.tree_format,'elements');
  assert.equal(q.max_elements,64);
  assert.equal(q.max_depth,10);
  assert.equal(q.display_only,undefined);
  assert.throws(()=>buildCuaSemanticWindowQuery({pid:0,windowId:1234}));
});

test('Cua semantic projection strips raw values, titles and action tokens',()=>{
  const raw={pid:42,window_id:1234,elements_complete:true,
    window_title:'PRIVATE WORLD NAME',
    tree_markdown:'PRIVATE SECRET',
    snapshot_id:'s0000000a',
    elements:[
      {element_index:1,role:'Button',label:'Play\nResume',enabled:true,
        actions:['invoke'],element_token:'s0000000a:1',
        value:'secret-value'},
      {element_index:2,role:'Edit',label:'private password',
        value:'private password',actions:['set_value']},
    ]};
  const result=projectCuaSemanticWindow(raw,{pid:42,windowId:1234});
  assert.equal(result.status,'UIA_ELEMENTS_READ_UNVERIFIED');
  assert.equal(result.elements[0].label,'Play Resume');
  assert.equal(result.elements[1].label,null);
  assert.equal(result.elements[1].role,'Edit');
  assert.equal(result.actionPermitted,false);
  assert.equal(result.screenshotCaptured,false);
  assert.equal(JSON.stringify(result).includes('PRIVATE'),false);
  assert.equal(JSON.stringify(result).includes('secret'),false);
  assert.equal(JSON.stringify(result).includes('s0000000a'),false);
  for(const bad of [
    {...raw,pid:43}, {...raw,window_id:1235},
    {...raw,screenshot_png_b64:'not a PNG'},
    {...raw,elements:[...raw.elements,...Array(63).fill(raw.elements[0])]},
  ]) assert.throws(()=>projectCuaSemanticWindow(bad,
      {pid:42,windowId:1234}),/CUA_UIA_READ_UNAVAILABLE/);
  assert.equal(projectCuaSemanticWindow({...raw,elements_complete:false},
    {pid:42,windowId:1234}).status,'UIA_PARTIAL_UNVERIFIED');
  assert.equal(projectCuaSemanticWindow({...raw,elements:[],degraded:true},
    {pid:42,windowId:1234}).status,'UIA_EMPTY_OR_UNAVAILABLE');
});

test('Cua semantic CLI uses existing binary and bounded tree-only daemon call',async()=>{
  const calls=[];
  const target={pid:42,windowId:1234};
  const result=await inspectCuaWindowElements(path.resolve('cua-driver.exe'),
    target,{platform:'win32',
      statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
      execute:async(exe,args,options)=>{
        calls.push({exe,args,options});
        return {stdout:JSON.stringify({
          pid:42,window_id:1234,elements_complete:true,
          elements:[{element_index:1,role:'Button',label:'Resume',
            actions:['invoke'],enabled:true}],
        })};
      },
    });
  assert.equal(calls.length,1);
  assert.deepEqual(calls[0].args.slice(0,2),
    ['call','get_window_state']);
  const q=JSON.parse(calls[0].args[2]);
  assert.equal(q.include_screenshot,false);
  assert.equal(q.max_elements,64);
  assert.equal(calls[0].options.shell,false);
  assert.equal(calls[0].options.maxBuffer,64*1024);
  assert.equal(result.elements[0].label,'Resume');
  await assert.rejects(()=>inspectCuaWindowElements(
    path.resolve('cua-driver.exe'),target,{platform:'win32',
      statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
      execute:async()=>({stdout:'NOT JSON'}),
    }),/CUA_UIA_READ_UNAVAILABLE/);
});

test('Cua reads only the stat-pinned number of PNG bytes with short reads',async()=>{
  const stat={isFile:()=>true,size:BigInt(png1x1.length),
    dev:1n,ino:2n,mtimeNs:100n,ctimeNs:100n};
  let reads=0;
  const handle={
    stat:async()=>stat,
    read:async(buffer,offset,length,position)=>{
      reads++;
      const count=Math.min(length,7);
      png1x1.copy(buffer,offset,position,position+count);
      return {bytesRead:count};
    },
  };
  const bytes=await readBoundedPng(handle);
  assert.deepEqual(bytes,png1x1);
  assert.ok(reads>1);
  assert.equal(inspectPng(bytes).width,1);
});

test('Cua rejects changed, oversized and truncated PNG files without output',async()=>{
  const stat={isFile:()=>true,size:BigInt(png1x1.length),
    dev:1n,ino:2n,mtimeNs:100n,ctimeNs:100n};
  let stats=0;
  const changed={
    stat:async()=>++stats===1?stat:{...stat,mtimeNs:101n},
    read:async(buffer,offset,length,position)=>{
      const count=Math.min(length,png1x1.length-position);
      png1x1.copy(buffer,offset,position,position+count);
      return {bytesRead:count};
    },
  };
  await assert.rejects(()=>readBoundedPng(changed),/changed/);
  let readCalled=false;
  const oversized={
    stat:async()=>({...stat,size:BigInt(MAX_PNG_BYTES+1)}),
    read:async()=>{readCalled=true;throw Error('must not read oversized file');},
  };
  await assert.rejects(()=>readBoundedPng(oversized),/budget/);
  assert.equal(readCalled,false);
  await assert.rejects(()=>readBoundedPng({
    stat:async()=>stat,read:async()=>({bytesRead:0}),
  }),/incomplete/);
});

test('PNG observation validates signature, dimensions and bounded source',()=>{
  const meta=inspectPng(png1x1);
  assert.equal(meta.width,1);
  assert.equal(meta.height,1);
  assert.match(meta.sha256,/^[a-f0-9]{64}$/);
  assert.throws(()=>inspectPng(Buffer.from('not-an-image')));
  const altered=Buffer.from(png1x1);
  altered.writeUInt32BE(99999,16);
  assert.throws(()=>inspectPng(altered),/dimensions/);
  assert.throws(()=>inspectPng(Buffer.alloc(MAX_PNG_BYTES+1)));
  const looksLikePng=Buffer.alloc(MAX_PNG_BYTES+1);
  png1x1.copy(looksLikePng); // A PNG signature cannot bypass the raw-size bound.
  assert.throws(()=>inspectPng(looksLikePng),/bounded PNG/);
});
