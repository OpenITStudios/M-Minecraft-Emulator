// For an independently trusted local execution host only. This is NOT a
// sandbox, generic tool dispatcher, new MCP session, or permission grant.
// The host creates ONE operation core, and attaches this facade to it.
import {REQUEST_LIMIT,boundedMcpText,publicToolError} from './agent-contract.mjs';

// Fixed composition routes: one declaration for SDK methods and compact MCP.
// The original operation core remains the only dispatch/validation authority.
const ROUTES = [
  ['profiles.inspect','inspect_profiles',[]],
  ['testing.context','get_test_context',['slots']],
  ['testing.probe','probe_qa',['slot','scope']],
  ['testing.inspectRun','probe_qa',['slot'],{inspectRun:true}],
  ['testing.readArchived','probe_qa',['archivedRunId'],{},['archivedRunId']],
  ['evidence.summarize','summarize_test_evidence',['slots']],
  ['evidence.read','read_game_evidence',['slot']],
  ['diagnostics.overview','get_diagnostics_overview',['run']],
  ['diagnostics.list','list_issue_candidates',['run','cursor','limit']],
  ['diagnostics.details','get_issue_details',['run','id']],
  ['diagnostics.evidence','query_diagnostic_evidence',['run','id','cursor','limit']],
  ['observation.state','get_multi_client_visual_state',['slots','since']],
  ['input.plan','plan_player_actions',['actions','includeTimeline','includeClientFrames']],
  ['input.assess','assess_input_route',['actions']],
  ['capabilities','agent_capabilities',[]],
];
const METHODS=Object.freeze(ROUTES.map(([method])=>method));

export function createCodeModeSurface(descriptors,callTool) {
  const allowedByTool=new Map();
  for(const [,tool,allowed,forced={}] of ROUTES) {
    const keys=allowedByTool.get(tool)??new Set();
    for(const key of [...allowed,...Object.keys(forced)])keys.add(key);
    allowedByTool.set(tool,keys);
  }
  const tools=descriptors.filter(tool=>allowedByTool.has(tool.name)).map(tool=>{
    const allowed=allowedByTool.get(tool.name);
    const schema=tool.inputSchema;
    const properties=Object.fromEntries(Object.entries(schema.properties??{})
      .filter(([key])=>allowed.has(key)));
    for(const key of allowed) {
      if(!Object.hasOwn(properties,key))throw new Error('Code Mode schema mismatch');
    }
    // probe_qa has three explicit branches. The core enforces their
    // mutually exclusive keys, and no restart/reservation/publication.
    if(tool.name==='probe_qa') {
      properties.inspectRun={const:true,type:'boolean'};
    }
    return {...tool,
      ...(tool.name==='probe_qa'?{description:'Canonical source-only Probe preparation/status, inspectRun=true or explicit archivedRunId. No restart, publication, reservation or game command dispatch.'}:{}),
      ...(tool.name==='agent_capabilities'?{description:'Read bounded local AI/QA capabilities and activity. No desktop publication or gameplay proof.'}:{}),
      inputSchema:{...schema,properties,
        ...(schema.required?{required:schema.required.filter(key=>allowed.has(key))}:{}),
        ...(tool.name==='probe_qa'?{oneOf:[
          {not:{anyOf:[{required:['inspectRun']},{required:['archivedRunId']}]}},
          {required:['inspectRun'],not:{anyOf:[{required:['archivedRunId']},{required:['scope']}]}},
          {required:['archivedRunId'],not:{anyOf:[{required:['slot']},{required:['scope']},{required:['inspectRun']}]}},
        ]}:{}),
        additionalProperties:false}};
  });
  if(tools.length!==allowedByTool.size)throw new Error('Code Mode tool catalog incomplete');
  const names=tools.map(tool=>tool.name).filter(name=>name!=='agent_capabilities');
  return {
    listTools:()=>tools,
    async callTool(name,args={}) {
      const allowed=allowedByTool.get(name);
      if(!allowed)throw new Error('Unknown tool');
      if(!args||typeof args!=='object'||Array.isArray(args)||
        Object.keys(args).some(key=>!allowed.has(key))) {
        throw new Error('Invalid tool arguments');
      }
      const result=await callTool(name,{...args});
      return name==='agent_capabilities'?{...result,available:[...names]}:result;
    },
  };
}

const MAX_CALLS=64;
const MAX_RESULT_BYTES=1024*1024;

export function createCodeModeClient(core) {
  let calls=0,resultBytes=0,busy=false,closed=false;
  // The host MUST close the facade at the end of a script. The core is
  // separately owned by the host, and may be shared with its MCP adapter.
  async function invoke(name,args,allowed,forced={},required=[]) {
    if(closed||core.isClosed()) throw new Error('CODE_MODE_CLOSED');
    if(busy) throw new Error('CODE_MODE_CALL_IN_PROGRESS');
    if(calls>=MAX_CALLS||resultBytes>=MAX_RESULT_BYTES) throw new Error('CODE_MODE_BUDGET_EXHAUSTED');
    busy=true;
    calls++;
    try {
      if(args===undefined)args={};
      if(!args||typeof args!=='object'||Array.isArray(args)||
        Object.getPrototypeOf(args)!==Object.prototype||
        Object.keys(args).some(key=>!allowed.includes(key))) {
        throw new Error('Invalid tool arguments');
      }
      const encoded=JSON.stringify(args);
      if(typeof encoded!=='string'||Buffer.byteLength(encoded,'utf8')>REQUEST_LIMIT) {
        throw new Error('Invalid tool arguments');
      }
      const input=JSON.parse(encoded);
      if(!input||typeof input!=='object'||Array.isArray(input)||
        Object.keys(input).some(key=>!allowed.includes(key))||
        required.some(key=>!Object.hasOwn(input,key))) {
        throw new Error('Invalid tool arguments');
      }
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      // Exact owner admission and typed result projections are unchanged.
      // A guest cannot specify a tool name, executable, source path, lease,
      // publication, restart, native reservation or arbitrary command.
      const result=await core.callTool(name,{...input,...forced});
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      const output=boundedMcpText(name==='agent_capabilities'?
        {...result,available:[...METHODS]}:result);
      resultBytes+=Buffer.byteLength(output,'utf8');
      if(resultBytes>MAX_RESULT_BYTES) throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
      return JSON.parse(output);
    } catch(error) {
      throw new Error(publicToolError(error));
    } finally {
      busy=false;
    }
  }
  const tester={};
  for(const [method,name,allowed,forced={},required=[]] of ROUTES) {
    const [namespace,member]=method.split('.');
    const call=args=>invoke(name,args,allowed,forced,required);
    if(member) (tester[namespace]??={})[member]=call;
    else tester[namespace]=call;
  }
  for(const value of Object.values(tester)) {
    if(typeof value==='object')Object.freeze(value);
  }
  Object.freeze(tester);
  return Object.freeze({tester,close:()=>{closed=true;}});
}
