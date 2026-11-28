#!/usr/bin/env node
// Single stdio adapter for playtesting and retained compatibility profiles.
import {pathToFileURL} from 'node:url';
import {createOperationCore,REQUEST_LIMIT,boundedMcpText,publicToolError,parseConfiguration} from './operation-core.mjs';
import {createCodeModeSurface} from './code-mode.mjs';
import {assertAgentSurface,mcpImageMime} from './agent-contract.mjs';
export * from './operation-core.mjs';

// The 2026-07-28 protocol removes initialize and requires per-request
// metadata. Do not advertise it through this retained legacy lifecycle.
const SUPPORTED_PROTOCOLS = new Set(['2025-06-18','2025-11-25']);

export function createMcpHandler(core,surface='legacy') {
  assertAgentSurface(surface);
  const route=surface==='legacy'?core:createCodeModeSurface(core.listTools(),core.callTool,surface);
  const pendingIds=new Set();
  async function handle(message) {
    if(core.isClosed())return null;
    if (!message || message.jsonrpc!=='2.0' || typeof message.method!=='string') return null;
    if (!Object.hasOwn(message,'id')) return null;
    const id=message.id;
    if(typeof id!=='string'&&!Number.isSafeInteger(id))return null;
    // A duplicate in-flight ID is invalid on this connection. Preserve the
    // original request and its only reply; never dispatch or queue it twice.
    // This is not a completed-request replay cache or a retry mechanism.
    if(pendingIds.has(id))return null;
    pendingIds.add(id);
    const result=(value)=>({jsonrpc:'2.0',id,result:value});
    const err=(code,message)=>({jsonrpc:'2.0',id,error:{code,message}});
    try {
      if (message.method==='initialize') {
        const declared=message.params?.protocolVersion;
        return result({protocolVersion:SUPPORTED_PROTOCOLS.has(declared)?declared:'2025-06-18',
          capabilities:{tools:{listChanged:false}},serverInfo:{name:'lazy-tester-agent',version:'0.1.0'}});
      }
      if (message.method==='ping') return result({});
      if (message.method==='tools/list') return result({tools:route.listTools()});
      if (message.method==='tools/call') {
        const name=message.params?.name;
        if (typeof name!=='string') return err(-32602,'Tool name is required');
        try {
          const data=await route.callTool(name,message.params?.arguments??{});
          if(core.isClosed())return null;
          if (name==='observe_client' || name==='read_visual_observation') {
            // Cached rolling observation returns metadata alone unless a
            // newer frame is explicitly requested by the model.
            const content=[{type:'text',text:boundedMcpText(data.metadata)}];
            const mimeType=mcpImageMime(data,surface==='playtest');
            if(mimeType!==null)content.push({type:'image',mimeType,data:data.imageBase64});
            return result({content,isError:false});
          }
          return result({content:[{type:'text',text:boundedMcpText(data)}],isError:false});
        } catch(error) {
          if(core.isClosed())return null;
          return result({content:[{type:'text',text:publicToolError(error)}],isError:true});
        }
      }
      return err(-32601,'Method not found');
    }catch {return err(-32603,'Internal error');}
    finally {pendingIds.delete(id);}
  }

  return {handle,callTool:route.callTool,shutdown:core.shutdown};
}

export function createHandler(config,dependencies) {
  const surface=assertAgentSurface(config.surface??'legacy',
    config.experimentalMc02InputEnabled||config.experimentalMultiClientInputEnabled);
  return createMcpHandler(createOperationCore(config,dependencies),surface);
}

const MAX_PENDING_REQUESTS=16;
const OUTPUT_STALL_MS=30000;

// One stdio connection, one bounded LF/CRLF frame, no request queue.
export function attachMcpStdio(handler,input,output) {
  const frame=Buffer.alloc(REQUEST_LIMIT+1); // Optional trailing CR.
  let used=0,oversized=false,pending=0,closed=false,stall=null;
  function close(abortOutput=false) {
    if(closed)return;
    closed=true;
    if(stall!==null)clearTimeout(stall);
    input.off('data',read);
    output.off('drain',drain);
    handler.shutdown();
    input.destroy();
    if(abortOutput||stall!==null)output.destroy();
  }
  function drain() {
    if(closed||stall===null)return;
    clearTimeout(stall);
    stall=null;
    input.resume();
  }
  async function dispatch(message) {
    try {
      const reply=await handler.handle(message);
      if(!reply||closed)return;
      if(!output.write(JSON.stringify(reply)+'\n')&&stall===null) {
        input.pause();
        // Do not keep an abandoned connection waiting on asynchronous drain.
        stall=setTimeout(()=>close(true),OUTPUT_STALL_MS);
      }
    }catch {close(true);}
    finally {pending--;}
  }
  function read(chunk) {
    for(let offset=0;offset<chunk.length&&!closed;) {
      const newline=chunk.indexOf(10,offset);
      const end=newline<0?chunk.length:newline;
      const length=end-offset;
      if(!oversized) {
        if(used+length>frame.length)oversized=true;
        else {chunk.copy(frame,used,offset,end);used+=length;}
      }
      if(newline<0)break;
      const size=used>0&&frame[used-1]===13?used-1:used;
      let message;
      if(!oversized&&size<=REQUEST_LIMIT) {
        try{message=JSON.parse(frame.toString('utf8',0,size));}catch{}
      }
      used=0;oversized=false;offset=end+1;
      if(!message||!Object.hasOwn(message,'id'))continue;
      // Independent Stop/other IDs stay live during normal concurrency.
      // A flooding peer is disconnected rather than queued or retried.
      if(pending>=MAX_PENDING_REQUESTS){close(true);break;}
      pending++;
      void dispatch(message);
    }
  }
  input.once('end',()=>close());
  input.once('close',()=>close());
  input.on('error',()=>close(true));
  output.once('close',()=>close(true));
  output.on('error',()=>close(true));
  output.on('drain',drain);
  input.on('data',read);
  if(input.readableEnded||input.destroyed||output.destroyed)close(true);
}

export function serve(config) {
  attachMcpStdio(createHandler(config),process.stdin,process.stdout);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {serve(parseConfiguration(process.argv.slice(2)));}
  catch(error){process.stderr.write(error.message+'\n');process.exitCode=2;}
}
