#!/usr/bin/env node
// Compatibility adapter only; the same core is usable by a trusted Code Mode host.
import {pathToFileURL} from 'node:url';
import readline from 'node:readline';
import {createOperationCore,REQUEST_LIMIT,MAX_MCP_IMAGE_CHARS,boundedMcpText,publicToolError,parseConfiguration} from './operation-core.mjs';
export * from './operation-core.mjs';

// The 2026-07-28 protocol removes initialize and requires per-request
// metadata. Do not advertise it through this retained legacy lifecycle.
const SUPPORTED_PROTOCOLS = new Set(['2025-06-18','2025-11-25']);

export function createMcpHandler(core) {
  async function handle(message) {
    if(core.isClosed())return null;
    if (!message || message.jsonrpc!=='2.0' || typeof message.method!=='string') return null;
    if (!Object.hasOwn(message,'id')) return null;
    const id=message.id;
    const result=(value)=>({jsonrpc:'2.0',id,result:value});
    const err=(code,message)=>({jsonrpc:'2.0',id,error:{code,message}});
    try {
      if (message.method==='initialize') {
        const declared=message.params?.protocolVersion;
        return result({protocolVersion:SUPPORTED_PROTOCOLS.has(declared)?declared:'2025-06-18',
          capabilities:{tools:{listChanged:false}},serverInfo:{name:'lazy-tester-agent',version:'0.1.0'}});
      }
      if (message.method==='ping') return result({});
      if (message.method==='tools/list') return result({tools:core.listTools()});
      if (message.method==='tools/call') {
        const name=message.params?.name;
        if (typeof name!=='string') return err(-32602,'Tool name is required');
        try {
          const data=await core.callTool(name,message.params?.arguments??{});
          if (name==='observe_client' || name==='read_visual_observation') {
            // Cached rolling observation returns metadata alone unless a
            // newer frame is explicitly requested by the model.
            const content=[{type:'text',text:boundedMcpText(data.metadata)}];
            if (typeof data.imageBase64==='string') {
              if(data.imageBase64.length>MAX_MCP_IMAGE_CHARS) {
                throw new Error('MCP_IMAGE_RESPONSE_TOO_LARGE');
              }
              content.push({type:'image',mimeType:'image/png',
                data:data.imageBase64});
            }
            return result({content,isError:false});
          }
          return result({content:[{type:'text',text:boundedMcpText(data)}],isError:false});
        } catch(error) {
          return result({content:[{type:'text',text:publicToolError(error)}],isError:true});
        }
      }
      return err(-32601,'Method not found');
    }catch {return err(-32603,'Internal error');}
  }

  return {handle,callTool:core.callTool,shutdown:core.shutdown};
}

export function createHandler(config,dependencies) {
  return createMcpHandler(createOperationCore(config,dependencies));
}

export function serve(config) {
  const handler=createHandler(config);
  const rl=readline.createInterface({input:process.stdin,crlfDelay:Infinity,terminal:false});
  let disconnected=false;
  rl.once('close',()=>{disconnected=true;handler.shutdown();});
  rl.on('line',async line=>{
    if (disconnected||Buffer.byteLength(line)>REQUEST_LIMIT) return;
    let message;
    try{message=JSON.parse(line);}catch{return;}
    try {
      const reply=await handler.handle(message);
      if (reply&&!disconnected) process.stdout.write(JSON.stringify(reply)+'\n');
    }catch { /* MCP stdout must remain parseable; failures are returned by handler. */ }
  });
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {serve(parseConfiguration(process.argv.slice(2)));}
  catch(error){process.stderr.write(error.message+'\n');process.exitCode=2;}
}
