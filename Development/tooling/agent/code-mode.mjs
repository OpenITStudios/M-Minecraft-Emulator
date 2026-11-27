// For an independently trusted local execution host only. This is NOT a
// sandbox, generic tool dispatcher, new MCP session, or permission grant.
// The host creates ONE operation core, and attaches this facade to it.
import {REQUEST_LIMIT,boundedMcpText,publicToolError} from './operation-core.mjs';

const MAX_CALLS=64;
const MAX_RESULT_BYTES=1024*1024;

export function createCodeModeClient(core) {
  let calls=0,resultBytes=0,busy=false,closed=false;
  // The host MUST close the facade at the end of a script. The core is
  // separately owned by the host, and may be shared with its MCP adapter.
  async function invoke(name,args,allowed,forced={}) {
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
        Object.keys(input).some(key=>!allowed.includes(key))) {
        throw new Error('Invalid tool arguments');
      }
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      // Exact owner admission and typed result projections are unchanged.
      // A guest cannot specify a tool name, executable, source path, lease,
      // publication, restart, native reservation or arbitrary command.
      const result=await core.callTool(name,{...input,...forced});
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      const output=boundedMcpText(result);
      resultBytes+=Buffer.byteLength(output,'utf8');
      if(resultBytes>MAX_RESULT_BYTES) throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
      return JSON.parse(output);
    } catch(error) {
      throw new Error(publicToolError(error));
    } finally {
      busy=false;
    }
  }
  const method=(name,allowed=[],forced={})=>args=>invoke(name,args,allowed,forced);
  const tester=Object.freeze({
    profiles:Object.freeze({inspect:method('inspect_profiles')}),
    testing:Object.freeze({
      context:method('get_test_context',['slots']),
      // Same volatile canonical Probe checkpoint. No restart, archive
      // publication, native reservation, or dispatch is admitted here.
      probe:method('probe_qa',['slot','scope']),
      inspectRun:method('probe_qa',['slot'],{inspectRun:true}),
      readArchived:args=>{
        if(!args||!Object.hasOwn(args,'archivedRunId')||
          typeof args.archivedRunId!=='string'||!/^[a-f0-9]{32}$/.test(args.archivedRunId)) {
          return Promise.reject(new Error('Invalid tool arguments'));
        }
        return invoke('probe_qa',args,['archivedRunId'],{archivedRunId:args.archivedRunId});
      },
    }),
    evidence:Object.freeze({
      summarize:method('summarize_test_evidence',['slots']),
      read:method('read_game_evidence',['slot']),
    }),
    diagnostics:Object.freeze({
      overview:method('get_diagnostics_overview',['run']),
      list:method('list_issue_candidates',['run','cursor','limit']),
      details:method('get_issue_details',['run','id']),
      evidence:method('query_diagnostic_evidence',['run','id','cursor','limit']),
    }),
    observation:Object.freeze({
      state:method('get_multi_client_visual_state',['slots','since']),
    }),
    input:Object.freeze({
      plan:method('plan_player_actions',['actions','includeTimeline','includeClientFrames']),
      assess:method('assess_input_route',['actions']),
    }),
    capabilities:method('agent_capabilities'),
  });
  return Object.freeze({tester,close:()=>{closed=true;}});
}
