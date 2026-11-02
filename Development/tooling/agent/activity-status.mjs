// Read-only UX activity projection over this SAME optional MCP process.
// No new worker, controller, game-input owner, QA executor or persistent store.
// These statuses describe local source observations, not control of Windows
// or successful Minecraft Education input / map gameplay.
const kinds=Object.freeze({AI:'AI_INPUT',OBS:'VISUAL_OBSERVATION',QA:'SKILLS_PROBE'});
const terminalStatus=new Set(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW','INCOMPLETE','BLOCKED']);
function slots(values) {
  if(!Array.isArray(values)||values.length>5||
    values.some(v=>!Number.isInteger(v)||v<1||v>5)||
    new Set(values).size!==values.length)throw new TypeError('Invalid local activity slots');
  return [...values].sort((a,b)=>a-b);
}
function add(items,kind,slot,status,attention=false,extra={}) {
  items.push(Object.freeze({kind,slot,status,
    importance:attention?'ATTENTION':'INFORMATION',
    ...extra}));
}
/**
 * The caller supplies ONLY already-owned volatile MCP observations.
 * Never accepts model-supplied claims that a PC, world or game is controlled.
 * A missing item means "not observed by this MCP", never "nothing is active".
 */
export function projectAgentActivity({
  mode,atMs,multiEnabled=false,legacyEnabled=false,
  multiSlots=[],legacyPending=false,uncertainSlots=[],
  startupReconcileSlots=[],probePlans=[],probeBusy=[],
  visualSamplers=[],
}={}) {
  if(!['human-plus-four-ai','five-ai'].includes(mode)||
     !Number.isSafeInteger(atMs)||atMs<1||
     ![multiEnabled,legacyEnabled,legacyPending].every(x=>typeof x==='boolean')||
     !Array.isArray(probePlans)||probePlans.length>5||
     !Array.isArray(visualSamplers)||visualSamplers.length>5)
    throw new TypeError('Invalid local activity snapshot');
  const active=slots(multiSlots),uncertain=slots(uncertainSlots),
    reconcile=slots(startupReconcileSlots),checking=slots(probeBusy);
  if(mode==='human-plus-four-ai' &&
    [...active,...uncertain,...reconcile].some(slot=>slot===1))
    throw new TypeError('Human MC01 may not be an AI input target');
  if(legacyEnabled&&multiEnabled)throw new TypeError('Conflicting input experiments');
  const items=[];
  // A lost/unknown native neutral receipt warrants attention, not "AI active".
  if(multiEnabled)for(const slot of uncertain) {
    add(items,kinds.AI,slot,reconcile.includes(slot)?
      'STARTUP_RECONCILIATION_REQUIRED':'NATIVE_RELEASE_UNCONFIRMED',true,
      {evidence:'LOCAL_MCP_RECOVERY_HOLD'});
  }
  if(multiEnabled)for(const slot of active.filter(v=>!uncertain.includes(v))) {
    add(items,kinds.AI,slot,'EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT',false,
      {evidence:'LOCAL_MCP_REQUEST_ONLY'});
  }
  if(legacyEnabled && legacyPending)
    add(items,kinds.AI,2,'EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT',false,
      {evidence:'LOCAL_MCP_REQUEST_ONLY'});

  // Only a running local sampler is observed here. No PNG bytes, HWNDs,
  // process IDs, screenshot contents or account/world identities are included.
  const seenSamplers=new Set();
  for(const sample of visualSamplers) {
    if(!sample||!Number.isInteger(sample.slot)||sample.slot<1||sample.slot>5||
       seenSamplers.has(sample.slot)||
       !['SAMPLER_ACTIVE_NO_RECENT_FRAME','SAMPLER_ACTIVE_RECENT_FRAME']
         .includes(sample.status))throw new TypeError('Invalid sampler activity');
    seenSamplers.add(sample.slot);
    if(mode==='human-plus-four-ai' && sample.slot===1)
      throw new TypeError('Human MC01 may not be an AI observation target');
    add(items,kinds.OBS,sample.slot,sample.status,false,
      {evidence:'LOCAL_SAMPLER_ONLY'});
  }
  const seenProbes=new Set();
  for(const p of probePlans) {
    if(!p||!Number.isInteger(p.slot)||p.slot<1||p.slot>5||
       seenProbes.has(p.slot)||
       (p.terminalStatus!==null&&p.terminalStatus!==undefined&&
         (typeof p.terminalStatus!=='string'||p.terminalStatus.length>60)))
      throw new TypeError('Invalid Probe activity');
    if(mode==='human-plus-four-ai'&&p.slot===1)
      throw new TypeError('Human MC01 may not be a QA target');
    seenProbes.add(p.slot);
    if(p.terminalStatus!==null&&p.terminalStatus!==undefined) {
      const status=terminalStatus.has(p.terminalStatus)?
        p.terminalStatus:'UNRECOGNIZED_TERMINAL_STATUS';
      const needsReview=status!=='PROBE_REPORTED_PASS';
      add(items,kinds.QA,p.slot,'PROBE_REPORT_AVAILABLE',needsReview,
        {evidence:'ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN',
          reportedOutcome:status});
    } else if(checking.includes(p.slot)) {
      add(items,kinds.QA,p.slot,'LOCAL_QA_REVIEW_IN_PROGRESS',false,
        {evidence:'MCP_PROCESS_ONLY'});
    } else {
      const preparedAt=typeof p.preparedAt==='string'?
        Date.parse(p.preparedAt):NaN;
      const expired=!Number.isFinite(preparedAt)||preparedAt>atMs||
        atMs-preparedAt>20*60*1000;
      add(items,kinds.QA,p.slot,expired?
        'CHECKPOINT_EXPIRED_OR_UNAVAILABLE':'AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE',
        expired,{evidence:'SOURCE_CHECKPOINT_ONLY'});
    }
  }
  for(const slot of checking.filter(v=>!seenProbes.has(v))) {
    if(mode==='human-plus-four-ai'&&slot===1)
      throw new TypeError('Human MC01 may not be a QA target');
    add(items,kinds.QA,slot,'LOCAL_QA_PREPARATION_IN_PROGRESS',false,
      {evidence:'MCP_PROCESS_ONLY'});
  }
  const order={AI_INPUT:0,SKILLS_PROBE:1,VISUAL_OBSERVATION:2};
  items.sort((a,b)=>a.slot-b.slot||order[a.kind]-order[b.kind]);
  return Object.freeze({
    schema:1,source:'CURRENT_MCP_PROCESS_MEMORY_ONLY',
    snapshotAt:new Date(atMs).toISOString(),
    displayHint:items.some(item=>item.importance==='ATTENTION')?
      'ATTENTION':items.length?'ACTIVITY':'HIDDEN_NO_LOCAL_SIGNAL',
    desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER',
    minecraftInputEffect:'NOT_VERIFIED',
    mapGameplayAcceptance:'NOT_TESTED',
    items:Object.freeze(items),
  });
}
