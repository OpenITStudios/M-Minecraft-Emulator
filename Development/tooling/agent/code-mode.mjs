// For an independently trusted local execution host only. This is NOT a
// sandbox, generic tool dispatcher, new MCP session, or permission grant.
// The host creates ONE operation core, and attaches this facade to it.
import {REQUEST_LIMIT,boundedMcpText,publicToolError,assertAgentSurface,MAX_MCP_IMAGE_CHARS,mcpImageMime} from './agent-contract.mjs';

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
  ['input.plan','plan_player_actions',['actions','groups','includeTimeline','includeClientFrames']],
  ['input.assess','assess_input_route',['actions','groups']],
  ['capabilities','agent_capabilities',[]],
];
// Both transports derive their admitted keys and named methods from these routes.
const PLAYTEST_EXCLUDE=new Set(['get_test_context','plan_player_actions','assess_input_route','read_game_evidence']);
const PLAYTEST_KEYS={
  'testing.probe':['restart'],
  'evidence.summarize':['window'],
  capabilities:['inspectProfiles'],
};
const PLAYTEST_ROUTES=[
  ...ROUTES.filter(([,tool])=>!PLAYTEST_EXCLUDE.has(tool))
    .map(([method,tool,allowed,...rest])=>
      [method,tool,[...allowed,...(PLAYTEST_KEYS[method]??[])],...rest]),
  ['profiles.launch','launch_profile',['slot'],{},['slot']],
  ['observation.capture','observe_client',['slot','width','height','afterFrame'],{},['slot']],
  ['observation.inspectUi','inspect_client_ui',['slot'],{},['slot']],
  ['observation.start','start_visual_observation',['slots','intervalMs','durationMs','width','height'],{},['slots']],
  ['observation.read','read_visual_observation',['slot','afterSequence','includeImage'],{},['slot']],
  ['observation.stop','stop_visual_observation',['slots'],{},['slots']],
];

// Playtest image responses need a small visual receipt, not the native
// identity/diagnostic transcript. The core retains and validates that full
// metadata before this projection. Existing legacy responses are unchanged.
function compactVisualReceipt(name,result) {
  const m=result.metadata;
  if(name==='observe_client')return {metadata:{
    schema:m.schema,slot:m.slot,provider:m.provider,mimeType:m.mimeType,
    width:m.width,height:m.height,capturedAtMs:m.capturedAtMs,
    frameRef:m.frameRef,hasNewImage:m.hasNewImage,
    background:m.background,screenshotStatus:m.screenshotStatus,
    gameInput:m.gameInput,
  },imageBase64:result.imageBase64};
  return {metadata:{
    schema:m.schema,slot:m.slot,status:m.status,active:m.active??false,
    freshFrameAvailable:m.freshFrameAvailable??false,sequence:m.sequence,
    hasNewImage:m.hasNewImage,frameAgeMs:m.frameAgeMs??null,
    ...(m.mimeType?{mimeType:m.mimeType}:{}),
    width:m.width??null,height:m.height??null,
    sourceFrameAtMs:m.sourceFrameAtMs??null,sourceTimeKind:m.sourceTimeKind??null,
    successfulSamples:m.successfulSamples??0,captureErrors:m.captureErrors??0,
    ...(m.errorCode?{errorCode:m.errorCode}:{}),
    gameInput:m.gameInput,
  },imageBase64:result.imageBase64};
}

export function createCodeModeSurface(descriptors,callTool,surface='code-mode') {
  assertAgentSurface(surface,descriptors.some(tool=>tool.name.includes('experimental')));
  if(surface==='legacy')throw new Error('Unknown agent surface');
  const playtest=surface==='playtest';
  const allowedByTool=new Map();
  for(const [,tool,allowed,forced={}] of (playtest?PLAYTEST_ROUTES:ROUTES)) {
    const keys=allowedByTool.get(tool)??new Set();
    for(const key of [...allowed,...Object.keys(forced)])keys.add(key);
    allowedByTool.set(tool,keys);
  }
  // A duplicate could otherwise hide a missing route behind the same count.
  const selected=descriptors.filter(tool=>allowedByTool.has(tool.name));
  if(selected.length!==allowedByTool.size||
     new Set(selected.map(tool=>tool.name)).size!==allowedByTool.size) {
    throw new Error('Code Mode tool catalog incomplete');
  }
  const tools=selected.map(tool=>{
    const allowed=allowedByTool.get(tool.name);
    // The canonical capability schema's mutually exclusive flags cannot
    // coexist in either projected surface; omit the inapplicable constraint.
    const {dependentRequired,not,...schema}=tool.inputSchema;
    const dependencies=Object.entries(dependentRequired??{}).filter(([key])=>allowed.has(key));
    const properties=Object.fromEntries(Object.entries(schema.properties??{})
      .filter(([key])=>allowed.has(key)));
    for(const key of allowed) {
      if(!Object.hasOwn(properties,key))throw new Error('Code Mode schema mismatch');
    }
    // probe_qa has three explicit branches. The core enforces their
    // mutually exclusive keys and each surface's actual admission.
    if(tool.name==='probe_qa') {
      properties.inspectRun={const:true,type:'boolean'};
    }
    return {...tool,
      ...(playtest&&tool.name==='observe_client'?{description:'One on-demand WGC WebP preview, default 640x360. Set width and height (160-1536) if needed. Pass earlier frameRef as afterFrame to omit identical images. No PNG fallback. Use read_visual_observation for an active sampler.'}:{}),
      ...(playtest&&tool.name==='start_visual_observation'?{description:'Optional bounded local WGC WebP sampler. Default 640x360, set both width and height (160-1536). Images only on requested reads.'}:{}),
      ...(playtest&&tool.name==='read_visual_observation'?{description:'Read a cached WebP frame only if newer than afterSequence (within the active sampler); metadata only with includeImage=false. After restarting the sampler, omit afterSequence.'}:{}),
      ...(playtest&&tool.name==='get_multi_client_visual_state'?{description:'Metadata only for existing local frame caches. No new capture; byte changes are not gameplay proof.'}:{}),
      ...(tool.name==='probe_qa'?{description:playtest?
        'Canonical source-only Probe preparation/status, explicit restart, inspectRun=true or explicit archivedRunId. No publication, reservation or game command dispatch.':
        'Canonical source-only Probe preparation/status, inspectRun=true or explicit archivedRunId. No restart, publication, reservation or game command dispatch.'}:{}),
      ...(tool.name==='agent_capabilities'?{description:playtest?
        'Read capabilities and configured sources; inspectProfiles=true adds one fresh native profile roster in the same call. No desktop publication or gameplay proof.':
        'Read bounded local AI/QA capabilities and activity. No desktop publication or gameplay proof.'}:{}),
      inputSchema:{...schema,properties,
        ...(not&&tool.name!=='agent_capabilities'?{not}:{}),
        ...(schema.required?{required:schema.required.filter(key=>allowed.has(key))}:{}),
        ...(dependencies.length?{dependentRequired:Object.fromEntries(dependencies
            .map(([key,required])=>[key,required.filter(member=>allowed.has(member))]))}:{}),
        ...(tool.name==='probe_qa'?{oneOf:[
          {not:{anyOf:[{required:['inspectRun']},{required:['archivedRunId']}]}},
          {required:['inspectRun'],not:{anyOf:[{required:['archivedRunId']},{required:['scope']},...(playtest?[{required:['restart']}]:[])]}},
          {required:['archivedRunId'],not:{anyOf:[{required:['slot']},{required:['scope']},{required:['inspectRun']},...(playtest?[{required:['restart']}]:[])]}},
        ]}:{}),
        additionalProperties:false}};
  });
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
      const forceWebp=playtest&&['observe_client','start_visual_observation','read_visual_observation'].includes(name);
      const result=await callTool(name,{...args,...(forceWebp?{imageFormat:'webp'}:{})});
      if(playtest&&['observe_client','read_visual_observation'].includes(name)) {
        return compactVisualReceipt(name,result);
      }
      return name==='agent_capabilities'?{...result,available:[...names],
        ...(playtest?{playtest:{
          surface:'playtest',
          autonomousGameplay:'BLOCKED_NO_VERIFIED_INPUT_BACKEND',
          close:'NOT_EXPOSED_DESKTOP_ADMISSION_REQUIRED',
          nativeCapture:'APP_CAPTURE_BINARY_APPROVAL_HELD',
          observation:{format:'image/webp',defaultWidth:640,defaultHeight:360,
            provider:result.configuredSources?.visual==='WGC'?'WGC_SOURCE_CANDIDATE':'WEBP_CAPTURE_REQUIRES_WGC'},
          evidenceAttribution:'OPERATOR_NAMED_UNVERIFIED',
        }}:{})}:result;
    },
  };
}

const MAX_CALLS=64;
const MAX_RESULT_BYTES=1024*1024;

export function createCodeModeClient(core,surface='code-mode') {
  assertAgentSurface(surface);
  if(surface==='legacy')throw new Error('Unknown agent surface');
  const playtest=surface==='playtest';
  const routes=playtest?PLAYTEST_ROUTES:ROUTES;
  const methods=routes.map(([method])=>method);
  // The opt-in Playtest facade uses exactly the existing wire projection.
  const dispatch=playtest?createCodeModeSurface(core.listTools(),core.callTool,surface).callTool:core.callTool;
  let calls=0,resultBytes=0,imageChars=0,busy=false,closed=false;
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
      // No arbitrary tool, executable, source path, lease or publication.
      const result=await dispatch(name,{...input,...forced});
      if(closed||core.isClosed())throw new Error('MCP_CONNECTION_CLOSED');
      const visual=playtest&&(name==='observe_client'||name==='read_visual_observation');
      const output=boundedMcpText(visual?result.metadata:name==='agent_capabilities'?
        {...result,available:[...methods]}:result);
      resultBytes+=Buffer.byteLength(output,'utf8');
      if(resultBytes>MAX_RESULT_BYTES) throw new Error('MCP_TEXT_RESPONSE_TOO_LARGE');
      const value=JSON.parse(output);
      if(!visual)return value;
      const mime=mcpImageMime(result,true);
      if(mime!==null) {
        imageChars+=result.imageBase64.length;
        if(imageChars>MAX_MCP_IMAGE_CHARS)throw new Error('MCP_IMAGE_RESPONSE_TOO_LARGE');
      }
      // Do not stringify/parse the large Base64 string a second time.
      return {metadata:value,imageBase64:mime===null?null:result.imageBase64};
    } catch(error) {
      throw new Error(publicToolError(error));
    } finally {
      busy=false;
    }
  }
  const tester={};
  for(const [method,name,allowed,forced={},required=[]] of routes) {
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
