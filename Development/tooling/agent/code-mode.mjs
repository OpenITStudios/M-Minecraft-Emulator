// Trusted host-side facade, not a sandbox or new permission grant.
// Use the exact MCP operation names and their canonical descriptors.
import {REQUEST_LIMIT,boundedMcpText,publicToolError,MAX_MCP_IMAGE_CHARS,mcpImageMime} from './agent-contract.mjs';
import {TOOL_LIST} from './tool-catalog.mjs';

const MAX_CALLS=64;
const MAX_RESULT_BYTES=1024*1024;

export function createAgentClient(core,...obsoleteSelection) {
  if(obsoleteSelection.length)throw new Error('Agent surface retired; use createAgentClient(core)');
  const catalog=core.listTools();
  // Preserve the trusted SDK's prior experimental-input exclusion.
  if(catalog.some(tool=>tool.name.includes('experimental'))) {
    throw new Error('Trusted SDK does not admit experimental input');
  }
  const names=new Set(catalog.map(tool=>tool.name));
  if(catalog.length!==TOOL_LIST.length||names.size!==catalog.length||
     TOOL_LIST.some(tool=>!names.has(tool.name))) {
    throw new Error('Agent tool catalog incomplete');
  }
  let calls=0,resultBytes=0,imageChars=0,busy=false,closed=false;
  async function invoke(tool,args={}) {
    if(closed||core.isClosed())throw new Error('CODE_MODE_CLOSED');
    if(busy)throw new Error('CODE_MODE_CALL_IN_PROGRESS');
    if(calls>=MAX_CALLS||resultBytes>=MAX_RESULT_BYTES)throw new Error('CODE_MODE_BUDGET_EXHAUSTED');
    busy=true;
    calls++;
    try {
      const properties=tool.inputSchema.properties??{};
      const required=tool.inputSchema.required??[];
      if(!args||typeof args!=='object'||Array.isArray(args)||
         Object.getPrototypeOf(args)!==Object.prototype||
         Object.keys(args).some(key=>!Object.hasOwn(properties,key)||args[key]===undefined)) {
        throw new Error('Invalid tool arguments');
      }
      const encoded=JSON.stringify(args);
      if(typeof encoded!=='string'||Buffer.byteLength(encoded,'utf8')>REQUEST_LIMIT) {
        throw new Error('Invalid tool arguments');
      }
      const input=JSON.parse(encoded);
      if(!input||typeof input!=='object'||Array.isArray(input)||
         Object.keys(input).some(key=>!Object.hasOwn(properties,key))||
         Object.keys(input).length!==Object.keys(args).length||
         required.some(key=>!Object.hasOwn(input,key))) {
        throw new Error('Invalid tool arguments');
      }
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      const result=await core.callTool(tool.name,input);
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      const visual=tool.name==='observe_client'||tool.name==='read_visual_observation';
      const output=boundedMcpText(visual?result.metadata:result);
      resultBytes+=Buffer.byteLength(output,'utf8');
      if(resultBytes>MAX_RESULT_BYTES)throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
      const value=JSON.parse(output);
      if(!visual)return value;
      const mime=mcpImageMime(result,input.imageFormat==='webp');
      if(mime!==null) {
        imageChars+=result.imageBase64.length;
        if(imageChars>MAX_MCP_IMAGE_CHARS)throw new Error('MCP_IMAGE_RESPONSE_TOO_LARGE');
      }
      return {metadata:value,imageBase64:mime===null?null:result.imageBase64};
    } catch(error) {
      throw new Error(publicToolError(error));
    } finally {busy=false;}
  }
  const tester=Object.fromEntries(catalog.map(tool=>[tool.name,args=>invoke(tool,args)]));
  return Object.freeze({tester:Object.freeze(tester),close:()=>{closed=true;}});
}
