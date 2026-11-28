// Canonical MCP descriptors. No transport, native calls or session state.
const ACTION_GROUPS={type:'array',minItems:1,maxItems:32,description:'Compact groups expand into at most 32 canonical actions; camera samples preserve total relative delta. Existing route limits apply after expansion.',items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:2000},hold:{type:'array',maxItems:8,uniqueItems:true,items:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use']}},look:{type:'object',properties:{deltaX:{type:'integer',minimum:-512,maximum:512},deltaY:{type:'integer',minimum:-512,maximum:512},steps:{type:'integer',minimum:1,maximum:32},curve:{type:'string',enum:['linear','smooth']}},required:['deltaX','deltaY'],additionalProperties:false}},required:['slot','atMs','durationMs'],anyOf:[{required:['hold']},{required:['look']}],additionalProperties:false}};
const ACTION_INPUT_ONE_OF=[{required:['actions']},{required:['groups']}];
export const TOOL_LIST=[
  {name:'start_visual_observation',description:'Start a time-limited LOCAL read-only frame sampler for 1-5 AI-owned Minecraft windows. No continuous images are sent to the model and no gameplay inputs occur.',inputSchema:{type:'object',properties:{slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,items:{type:'integer',minimum:1,maximum:5}},intervalMs:{type:'integer',minimum:400,maximum:5000},durationMs:{type:'integer',minimum:5000,maximum:90000},imageFormat:{type:'string',enum:['png','webp']},width:{type:'integer',minimum:160,maximum:1536},height:{type:'integer',minimum:160,maximum:1536}},dependentRequired:{width:['height'],height:['width']},required:['slots'],additionalProperties:false}},
  {name:'get_multi_client_visual_state',
    description:'Read a single compact METADATA-ONLY view of existing cached visual frames across 1-5 AI clients. No screenshot is taken, no pixels are transmitted, and changed PNG bytes are not gameplay proof. Read individual frames explicitly to make model decisions.',
    inputSchema:{type:'object',properties:{
      slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
        items:{type:'integer',minimum:1,maximum:5}},
      since:{type:'array',minItems:1,maxItems:5,items:{type:'object',
        properties:{
          slot:{type:'integer',minimum:1,maximum:5},
          sequence:{type:'integer',minimum:0},
          samples:{type:'integer',minimum:0},
          collectedAt:{type:'string'},
          sourceFrameAtMs:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
          sourceTimeKind:{enum:['WGC_READBACK_COMPLETED','CUA_QUERY_STARTED',null]},
          producerSequence:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
          captureIdentity:{type:'object',properties:{
            pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},
            sessionId:{type:'integer',minimum:0},createdFileTime:{type:'string'},
          },required:['pid','windowId','sessionId','createdFileTime'],
            additionalProperties:false},
        },required:['slot','sequence','samples','collectedAt',
          'sourceFrameAtMs','sourceTimeKind','producerSequence','captureIdentity'],
        additionalProperties:false}},
    },additionalProperties:false}},
    {name:'read_visual_observation',description:'Read an existing cached frame; optional model-transcribed Show Coordinates text is screened against exact existing frame identity without OCR or new screenshot.',inputSchema:{type:'object',properties:{
    slot:{type:'integer',minimum:1,maximum:5},afterSequence:{type:'integer',minimum:0},includeImage:{type:'boolean'},imageFormat:{type:'string',enum:['png','webp']},
    screenReview:{type:'object',properties:{
      frameSequence:{type:'integer',minimum:1},sourceFrameAtMs:{type:'integer',minimum:1},
      identity:{type:'object',properties:{
        pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},sessionId:{type:'integer',minimum:0},
        createdFileTime:{type:'string',pattern:'^[0-9]{1,20}$'},
      },required:['pid','windowId','sessionId','createdFileTime'],additionalProperties:false},
      recognizedLine:{type:'string',minLength:1,maxLength:128},
    },required:['frameSequence','sourceFrameAtMs','identity','recognizedLine'],additionalProperties:false},
  },required:['slot'],additionalProperties:false}},
  {name:'stop_visual_observation',description:'Stop one or more bounded visual samplers and immediately drop their in-memory image caches.',inputSchema:{type:'object',properties:{slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,items:{type:'integer',minimum:1,maximum:5}}},required:['slots'],additionalProperties:false}},
  {name:'inspect_client_ui',description:'Read up to 64 UI Automation elements from ONE verified AI-owned Minecraft window via configured Cua Driver. No screenshot, game input, foreground actions, credentials, or action tokens. Incomplete/empty trees remain UNVERIFIED.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'assess_client_background_action',description:'SOURCE-ONLY READ-ONLY: native-fenced UIA Button check; ALWAYS BACKGROUND_UNAVAILABLE, including buttons advertising Invoke. semanticCandidate=true is informational only: upstream Cua exposes no verified no-fallback Invoke-only route, app-owned postcondition or focus safety. Never dispatches input.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},elementIndex:{type:'integer',minimum:0,maximum:5000},action:{type:'string',enum:['invoke']}},required:['slot','elementIndex','action'],additionalProperties:false}},
  {name:'observe_client',description:'Read-only screenshot of one AI-owned native Minecraft window via the configured provider; rejects a replaced or stale window, never takes foreground or sends input.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},imageFormat:{type:'string',enum:['png','webp']},width:{type:'integer',minimum:160,maximum:1536},height:{type:'integer',minimum:160,maximum:1536},afterFrame:{type:'string',pattern:'^[a-f0-9]{64}$',description:'Optional earlier one-shot WebP frameRef; suppress only identical bytes from the same target and size.'}},dependentRequired:{width:['height'],height:['width']},required:['slot'],additionalProperties:false}},
  {name:'get_computer_use_target',description:'Resolve one AI-owned native Minecraft PID/HWND for read-only Cua Driver window capture without switching Windows focus. Does NOT capture pixels or control game input.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'revalidate_computer_use_target',description:'Check a previously resolved exact PID/HWND/birth identity against current retained-controller observation. Never silently retarget another game.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},createdFileTime:{type:'string',pattern:'^[0-9]{1,20}$'}},required:['slot','pid','windowId','createdFileTime'],additionalProperties:false}},
  {name:'get_test_context',description:'One bounded read-only handoff with current five native profiles, peer-host guidance and relevant game evidence. No game control or world success is inferred.',inputSchema:{type:'object',properties:{slots:{type:'array',items:{type:'integer',minimum:1,maximum:5},minItems:1,maxItems:5,uniqueItems:true}},additionalProperties:false}},
  {name:'agent_capabilities',description:'Read local AI/QA capabilities and activity. Optional inspectProfiles=true adds one fresh native profile roster; optional publishToDesktop=true mirrors bounded status (not together). No game login, world readiness or gameplay proof.',inputSchema:{type:'object',properties:{publishToDesktop:{type:'boolean'},inspectProfiles:{type:'boolean'}},not:{required:['publishToDesktop','inspectProfiles'],properties:{publishToDesktop:{const:true},inspectProfiles:{const:true}}},additionalProperties:false}},
  {name:'inspect_profiles',description:'Read five profile states from the authenticated native Lazy Tester controller (not game login or world readiness).',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'launch_profile',description:'Explicitly request native Minecraft launch for an AI-controlled profile. Human MC01 cannot be launched in hybrid mode.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'plan_peer_session',description:'Inspect native profiles and provide conservative human/AI host-and-join guidance. No world control or join is performed.',inputSchema:{type:'object',properties:{},additionalProperties:false}},
  {name:'plan_player_actions',description:'Validate concurrent player actions or compact groups (choose one); grouped preview defaults to a compact receipt. PREVIEW ONLY: no Minecraft input.',inputSchema:{type:'object',properties:{actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},kind:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use','look_left','look_right','look_relative','wait']},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:5000},deltaX:{type:'integer',minimum:-512,maximum:512},deltaY:{type:'integer',minimum:-512,maximum:512}},required:['slot','kind','atMs','durationMs'],additionalProperties:false}},includeTimeline:{type:'boolean'},includeClientFrames:{type:'boolean'},groups:ACTION_GROUPS},oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false}},
  {name:'assess_input_route',description:'Read-only native identity check for AI gameplay input: bind validated multi-client action timeline to real MC HWND/PID; ALWAYS reports no verified independent game-input backend and NEVER sends input.',inputSchema:{type:'object',properties:{actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},kind:{type:'string',enum:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use','look_left','look_right','look_relative','wait']},atMs:{type:'integer',minimum:0,maximum:30000},durationMs:{type:'integer',minimum:50,maximum:5000},deltaX:{type:'integer',minimum:-512,maximum:512},deltaY:{type:'integer',minimum:-512,maximum:512}},required:['slot','kind','atMs','durationMs'],additionalProperties:false}},groups:ACTION_GROUPS},oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false}},
  {name:'summarize_test_evidence',description:'Read up to 20 bounded team [ANALYTICS] and [GAME_RESULTS] records per profile. Only sanitized summaries reach the model; no game identity or gameplay PASS is inferred.',inputSchema:{type:'object',properties:{slots:{type:'array',items:{type:'integer',minimum:1,maximum:5},minItems:1,maxItems:5,uniqueItems:true},window:{type:'string',enum:['begin','read'],description:'With explicit slots, begin a local capture baseline or read appended complete records since it. No game-session attribution.'}},dependentRequired:{window:['slots']},additionalProperties:false}},
  {name:'read_game_evidence',description:'Read one compact allowlisted 1-20 record projection of team analytics/results or optional generic diagnostics. No raw game JSON, player record IDs, credentials or gameplay PASS.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5}},required:['slot'],additionalProperties:false}},
  {name:'probe_qa',description:'Canonical Testing entry. inspectRun=true reads the original controller per-slot reservation, bounded parent status and optional Data/Testing archive reconciliation without starting a run, taking ownership or proving AI liveness. coordinateNative=true reserves a cooperating run; archivedRunId reads source-only immutable summaries. No automatic game command, operator approval, takeover or verified map PASS.',inputSchema:{type:'object',properties:{slot:{type:'integer',minimum:1,maximum:5},inspectRun:{type:'boolean'},restart:{type:'boolean'},scope:{type:'string',enum:['full','smoke','affected']},publishToDesktop:{type:'boolean'},coordinateNative:{type:'boolean'},archivedRunId:{type:'string',pattern:'^[a-f0-9]{32}$'}},additionalProperties:false}},
  // Source-only local-first diagnostics: explicitly imported immutable evidence.
  // No raw log, filesystem path, or automatic Minecraft activity is exposed.
  {name:'get_diagnostics_overview',
    description:'Read a tiny LOCAL diagnostic summary or up to 10 run IDs. No gameplay success or raw event dump. Import is never triggered over MCP.',
    inputSchema:{type:'object',properties:{
      run:{type:'string',pattern:'^[a-z][a-z0-9._-]{0,47}$'},
    },additionalProperties:false}},
  {name:'list_issue_candidates',
    description:'Return up to 10 grouped UNVERIFIED local issue candidates and a pagination cursor. No raw logs.',
    inputSchema:{type:'object',properties:{
      run:{type:'string',pattern:'^[a-z][a-z0-9._-]{0,47}$'},
      cursor:{type:'integer',minimum:0},limit:{type:'integer',minimum:1,maximum:10},
    },required:['run'],additionalProperties:false}},
  {name:'get_issue_details',
    description:'Read one selected UNVERIFIED issue with at most 8 curated samples. Never returns full source logs.',
    inputSchema:{type:'object',properties:{
      run:{type:'string',pattern:'^[a-z][a-z0-9._-]{0,47}$'},
      id:{type:'string',pattern:'^[a-f0-9]{16}$'},
    },required:['run','id'],additionalProperties:false}},
  {name:'query_diagnostic_evidence',
    description:'Read at most 8 allowlisted evidence records for a selected issue from LOCAL archive; paginated and no arbitrary file paths.',
    inputSchema:{type:'object',properties:{
      run:{type:'string',pattern:'^[a-z][a-z0-9._-]{0,47}$'},
      id:{type:'string',pattern:'^[a-f0-9]{16}$'},
      cursor:{type:'integer',minimum:0},limit:{type:'integer',minimum:1,maximum:8},
    },required:['run','id'],additionalProperties:false}},
];


// Deliberately NOT in TOOL_LIST: omitted from tools/list in the normal mode.
// Only an explicitly opted-in development MCP advertises it.
const EXPERIMENTAL_MC02_TOOL={
  name:'send_experimental_mc02_input',
  description:'EXPERIMENTAL/OPT-IN: send one <=500ms MC02-only virtual Win32 polling state. MC01 stays human foreground. This does not prove Minecraft gameplay or support Raw Input/GameInput.',
  inputSchema:{type:'object',properties:{
    slot:{type:'integer',const:2},
    buttons:{type:'array',maxItems:8,uniqueItems:true,
      items:{type:'string',enum:['move_forward','move_backward','strafe_left',
        'strafe_right','jump','sneak','attack','use']}},
    lookDX:{type:'integer',minimum:-127,maximum:127},
    lookDY:{type:'integer',minimum:-127,maximum:127},
    ttlMs:{type:'integer',minimum:1,maximum:500},
  },required:['slot','buttons'],additionalProperties:false},
};
// These are intentionally opt-in ONLY, using the same MC02 native shared-state
// writer. No provider for MC03-MC05 and no change to the normal tools list.
const EXPERIMENTAL_MC02_SEQUENCE_TOOLS=[
  {name:'run_experimental_mc02_actions',
    description:'EXPERIMENTAL MC02 only: execute a short timed action sequence with held-key heartbeat and best-effort neutral release. Native state ACK, NOT verified Minecraft gameplay.',
    inputSchema:{type:'object',properties:{
      slot:{type:'integer',const:2},
      actions:{type:'array',minItems:1,maxItems:12,
        items:{type:'object',properties:{
          slot:{type:'integer',const:2},
          kind:{type:'string',enum:['move_forward','move_backward',
            'strafe_left','strafe_right','jump','sneak','attack','use','look_relative']},
          atMs:{type:'integer',minimum:0,maximum:3500},
          durationMs:{type:'integer',minimum:50,maximum:2000},
          deltaX:{type:'integer',minimum:-127,maximum:127},
          deltaY:{type:'integer',minimum:-127,maximum:127},
        },required:['slot','kind','atMs','durationMs'],additionalProperties:false}},
      groups:ACTION_GROUPS,
    },required:['slot'],oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false}},
  {name:'stop_experimental_mc02_actions',
    description:'Cancel an active experimental MC02 sequence; when idle, request an exact-owner native neutral-state write. Transport ACK only, not gameplay release proof.',
    inputSchema:{type:'object',properties:{slot:{type:'integer',const:2}},
      required:['slot'],additionalProperties:false}},
];

// Registered ONLY after a separate developer opt-in. Native DLL/Cargo
// features remain absent from the ordinary packaged application.
const EXPERIMENTAL_MULTI_CLIENT_TOOLS=[
  {name:'run_experimental_multi_client_actions',
    description:'EXPERIMENTAL: dispatch a <=3.5s cross-slot action timeline to 1-5 opted-in AI game clients in one bounded native CLI. Transport acknowledgement only, NOT verified Minecraft gameplay.',
    inputSchema:{type:'object',properties:{
      actions:{type:'array',minItems:1,maxItems:32,items:{type:'object',properties:{
        slot:{type:'integer',minimum:1,maximum:5},
        kind:{type:'string',enum:['move_forward','move_backward',
          'strafe_left','strafe_right','jump','sneak','attack','use','look_relative']},
        atMs:{type:'integer',minimum:0,maximum:3500},
        durationMs:{type:'integer',minimum:50,maximum:2000},
        deltaX:{type:'integer',minimum:-127,maximum:127},
        deltaY:{type:'integer',minimum:-127,maximum:127},
      },required:['slot','kind','atMs','durationMs'],additionalProperties:false}},
      groups:ACTION_GROUPS,
    },oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false}},
  {name:'stop_experimental_multi_client_actions',
    description:'Cancel the whole active cross-slot experiment if a requested slot participates, or request exact-owner neutral writes for listed idle AI clients. Never sends global keyboard/mouse input.',
    inputSchema:{type:'object',properties:{
      slots:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
        items:{type:'integer',minimum:1,maximum:5}},
    },required:['slots'],additionalProperties:false}},
];

// This opt-in tool only binds a RECENT per-slot frame snapshot to the
// existing native batch call. It never makes AI decisions or bypasses the
// original game controller's independent input admission checks.
const EXPERIMENTAL_OBSERVED_TEAM_TOOL={
  name:'run_experimental_observed_actions',
  description:'EXPERIMENTAL: require an explicitly delivered current PNG and exact recent visual baseline for EVERY selected AI slot, then reuse the original native batch. Follow with get_multi_client_visual_state to inspect newer samples; PNG delivery is not proof of model review or gameplay success.',
  inputSchema:{type:'object',properties:{
    actions:EXPERIMENTAL_MULTI_CLIENT_TOOLS[0].inputSchema.properties.actions,
    groups:ACTION_GROUPS,
    basis:{type:'array',minItems:1,maxItems:5,uniqueItems:true,
      items:{type:'object',properties:{
        slot:{type:'integer',minimum:1,maximum:5},
        sequence:{type:'integer',minimum:1},
        samples:{type:'integer',minimum:1},
        collectedAt:{type:'string'},
        sourceFrameAtMs:{type:'integer',minimum:1},
        sourceTimeKind:{type:'string',
          enum:['WGC_READBACK_COMPLETED','CUA_QUERY_STARTED']},
        producerSequence:{anyOf:[{type:'integer',minimum:1},{type:'null'}]},
        captureIdentity:{type:'object',properties:{
          pid:{type:'integer',minimum:1},windowId:{type:'integer',minimum:1},
          sessionId:{type:'integer',minimum:0},
          createdFileTime:{type:'string',pattern:'^[0-9]{1,20}$'},
        },required:['pid','windowId','sessionId','createdFileTime'],
          additionalProperties:false},
      },required:['slot','sequence','samples','collectedAt',
        'sourceFrameAtMs','sourceTimeKind','producerSequence','captureIdentity'],
        additionalProperties:false}},
  },required:['basis'],oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false},
};

// Optional read-only result classification. Input receipt and AI review are
// caller-supplied; neither is independently authenticated as gameplay proof.
const EXPERIMENTAL_TEAM_FEEDBACK_TOOL={
  name:'evaluate_experimental_multi_client_outcomes',
  description:'EXPERIMENTAL READ-ONLY: classify per-client movement, camera, attack and interaction reports against exact newer same-window frame metadata. A model-reported visual change is NEVER verified Minecraft movement or attack success.',
  inputSchema:{type:'object',properties:{
    actions:EXPERIMENTAL_MULTI_CLIENT_TOOLS[0].inputSchema.properties.actions,
    groups:ACTION_GROUPS,
    receipt:{type:'object'},
    after:{type:'array',minItems:1,maxItems:5,items:{anyOf:[
      EXPERIMENTAL_OBSERVED_TEAM_TOOL.inputSchema.properties.basis.items,
      {type:'null'},
    ]}},
    reviews:{type:'array',maxItems:20,items:{type:'object',properties:{
      slot:{type:'integer',minimum:1,maximum:5},
      category:{type:'string',enum:['movement','camera','attack','interaction']},
      assessment:{type:'string',enum:[
        'VISIBLE_CHANGE_REPORTED','NO_VISIBLE_CHANGE_REPORTED','UNDETERMINED',
      ]},
      cue:{type:'string',maxLength:160},
    },required:['slot','category','assessment'],additionalProperties:false}},
  },required:['receipt','after'],oneOf:ACTION_INPUT_ONE_OF,additionalProperties:false},
};

export function toolsForMode(experimental,multi) {
  if(experimental) return [...TOOL_LIST,EXPERIMENTAL_MC02_TOOL,...EXPERIMENTAL_MC02_SEQUENCE_TOOLS];
  if(multi) return [...TOOL_LIST,...EXPERIMENTAL_MULTI_CLIENT_TOOLS,
    EXPERIMENTAL_OBSERVED_TEAM_TOOL,EXPERIMENTAL_TEAM_FEEDBACK_TOOL];
  return TOOL_LIST;
}

