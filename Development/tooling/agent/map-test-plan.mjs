// Source-only Map Test Contract and conservative selection for the ONE existing Probe QA MCP.
// No in-game runner, dynamic code, game input, filesystem mutation or gameplay PASS.
import {createHash} from 'node:crypto';
import {lstat,readFile} from 'node:fs/promises';
import path from 'node:path';

const MAX_BYTES=96*1024;
const MAX_REQUIREMENTS=64;
const MAX_CHANGED_PATHS=128;
const TOKEN=/^[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*){0,5}$/;
const MAP=/^[a-z][a-z0-9-]{1,47}$/;
const REVISION=/^[a-z0-9][a-z0-9._-]{0,63}$/;
const SHA=/^[a-f0-9]{40}$/;
const EVIDENCE=new Set(['team_analytics','team_game_results','map_state',
  'confirmed_action','visual_frame']);
const OPS=new Set(['equals','unchanged','increases','decreases','exists']);
// Static source capability map, NEVER a live-world readiness assertion.
export const MAP_QA_CAPABILITY_STATES=Object.freeze({
  team_analytics:'READER_SOURCE_ONLY_WORLD_ORIGIN_NOT_VERIFIED',
  team_game_results:'READER_SOURCE_ONLY_WORLD_ORIGIN_NOT_VERIFIED',
  map_state:'MAP_ADAPTER_NOT_IMPLEMENTED',
  confirmed_action:'PLAYER_ACTION_DELIVERY_NOT_VERIFIED',
  visual_frame:'OPTIONAL_CAPTURE_NOT_RUNTIME_VERIFIED',
});

function refuse(){throw new Error('MAP_TEST_CONTRACT_REVIEW_REQUIRED');}
function object(value){
  return value!==null&&typeof value==='object'&&!Array.isArray(value);
}
function keys(value,required,optional=[]){
  return object(value)&&required.every(k=>Object.hasOwn(value,k))&&
    Object.keys(value).every(k=>required.includes(k)||optional.includes(k));
}
function token(value){
  return typeof value==='string'&&value.length<=72&&TOKEN.test(value);
}
function sourcePath(value,{prefix=false}={}){
  if(typeof value!=='string'||value.length<1||value.length>160||
     value.includes('\\')||value.startsWith('/')||
     !/^[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)*\/?$/.test(value))return false;
  const segments=value.split('/').filter(Boolean);
  if(segments.some(s=>s==='.'||s==='..'))return false;
  return prefix||!value.endsWith('/');
}
function unique(items){return new Set(items).size===items.length;}
function list(value,max,predicate,{empty=false}={}){
  return Array.isArray(value)&&value.length<=(max)&&
    (empty||value.length>0)&&unique(value)&&value.every(predicate);
}
function expected(value){
  if(!keys(value,['operator'],['value'])||!OPS.has(value.operator))return false;
  const hasValue=Object.hasOwn(value,'value');
  if(value.operator!=='equals')return !hasValue;
  return hasValue&&(typeof value.value==='boolean'||
    typeof value.value==='string'&&/^[A-Za-z][A-Za-z0-9_.:-]{0,71}$/.test(value.value)||
    Number.isSafeInteger(value.value)&&Math.abs(value.value)<=1000000000);
}
function requirement(value){
  if(!keys(value,['id','given','action','observe','expect','evidence','affects'],
    ['required','smoke','dependsOn'])||
    !token(value.id)||!token(value.given)||!token(value.action)||
    !token(value.observe)||!expected(value.expect)||
    (value.required!==undefined&&typeof value.required!=='boolean')||
    (value.smoke!==undefined&&typeof value.smoke!=='boolean')||
    !list(value.evidence,5,x=>EVIDENCE.has(x))||
    !list(value.affects,16,x=>sourcePath(x,{prefix:true}))||
    (value.dependsOn!==undefined&&!list(value.dependsOn,16,token,{empty:true})))refuse();
  // A test that proposes an action cannot silently PASS without proof of action.
  if(value.action!=='none'&&!value.evidence.includes('confirmed_action'))refuse();
  // Keep contract as data. It never authorizes the action in any environment.
  return Object.freeze({...value,required:value.required??true,
    smoke:value.smoke??false,dependsOn:Object.freeze([...(value.dependsOn??[])]),
    evidence:Object.freeze([...value.evidence]),
    affects:Object.freeze([...value.affects]),
    expect:Object.freeze({...value.expect})});
}
export function validateMapTestContract(value){
  if(!keys(value,['schema','map','revision','requirements'])||
    value.schema!==1||typeof value.map!=='string'||!MAP.test(value.map)||
    typeof value.revision!=='string'||!REVISION.test(value.revision)||
    !Array.isArray(value.requirements)||
    value.requirements.length<1||value.requirements.length>MAX_REQUIREMENTS)refuse();
  const requirements=value.requirements.map(requirement);
  const byId=new Map(requirements.map(r=>[r.id,r]));
  if(byId.size!==requirements.length)refuse();
  for(const r of requirements){
    if(r.dependsOn.includes(r.id)||r.dependsOn.some(id=>!byId.has(id)))refuse();
  }
  const visiting=new Set(),complete=new Set();
  function visit(id){
    if(visiting.has(id))refuse();
    if(complete.has(id))return;
    visiting.add(id);
    for(const dep of byId.get(id).dependsOn)visit(dep);
    visiting.delete(id);complete.add(id);
  }
  for(const id of byId.keys())visit(id);
  return Object.freeze({schema:1,map:value.map,revision:value.revision,
    requirements:Object.freeze(requirements)});
}
function matchPath(changed,selector){
  return selector.endsWith('/')?changed.startsWith(selector):changed===selector;
}
function verifiedShape(changeSet){
  return keys(changeSet,['baseSha','headSha','paths'])&&
    typeof changeSet.baseSha==='string'&&typeof changeSet.headSha==='string'&&
    SHA.test(changeSet.baseSha)&&SHA.test(changeSet.headSha)&&
    changeSet.baseSha!==changeSet.headSha&&
    list(changeSet.paths,MAX_CHANGED_PATHS,x=>sourcePath(x));
}
/**
 * This pure planner accepts an asserted diff for source-only previews. It does
 * NOT authenticate the caller's Git diff; the MCP does not expose that input.
 * Missing/ambiguous changes always select FULL, never an optimistic subset.
 */
export function planMapTests(contract,{scope='full',changeSet=null}={}){
  const validated=validateMapTestContract(contract);
  if(!['full','smoke','affected'].includes(scope))refuse();
  const byId=new Map(validated.requirements.map(r=>[r.id,r]));
  const reverse=new Map(validated.requirements.map(r=>[r.id,[]]));
  for(const r of validated.requirements){
    for(const dep of r.dependsOn)reverse.get(dep).push(r.id);
  }
  let effectiveScope=scope,reason='EXPLICIT_SCOPE',seeds=[];
  if(scope==='full')seeds=[...byId.keys()];
  else if(scope==='smoke'){
    seeds=validated.requirements.filter(r=>r.smoke).map(r=>r.id);
    if(!seeds.length){
      effectiveScope='full';reason='NO_SMOKE_COVERAGE_FULL_FALLBACK';
      seeds=[...byId.keys()];
    }
  }else{
    if(!verifiedShape(changeSet)){
      effectiveScope='full';reason='NO_TRUSTED_DIFF_FULL_FALLBACK';
      seeds=[...byId.keys()];
    }else{
      const matched=validated.requirements.filter(r=>
        changeSet.paths.some(p=>r.affects.some(selector=>matchPath(p,selector))));
      if(!matched.length||changeSet.paths.some(p=>
          !validated.requirements.some(r=>r.affects.some(s=>matchPath(p,s))))){
        effectiveScope='full';reason='UNMAPPED_SOURCE_CHANGE_FULL_FALLBACK';
        seeds=[...byId.keys()];
      }else{
        reason='CALLER_ASSERTED_DIFF_NOT_INDEPENDENTLY_VERIFIED';
        seeds=matched.map(r=>r.id);
      }
    }
  }
  // Only affected changes propagate to dependents. Add prerequisites AFTER
  // impact closure so a shared dependency does not pull in unrelated tests.
  const impacted=new Set(seeds),impactQueue=[...seeds];
  if(scope==='affected'&&effectiveScope==='affected'){
    while(impactQueue.length){
      for(const dependent of reverse.get(impactQueue.shift())){
        if(!impacted.has(dependent)){
          impacted.add(dependent);impactQueue.push(dependent);
        }
      }
    }
  }
  const selected=new Set(impacted),prerequisites=[...impacted];
  while(prerequisites.length){
    for(const dep of byId.get(prerequisites.shift()).dependsOn){
      if(!selected.has(dep)){selected.add(dep);prerequisites.push(dep);}
    }
  }
  const sorted=[],seen=new Set();
  function order(id){
    if(seen.has(id)||!selected.has(id))return;
    for(const dep of byId.get(id).dependsOn)order(dep);
    seen.add(id);sorted.push(id);
  }
  for(const id of [...selected].sort())order(id);
  const evidence=[...new Set(sorted.flatMap(id=>byId.get(id).evidence))].sort();
  // An approved map contract is the only source of expected behavior.
  // Emit bounded per-check summaries for an external model to REVIEW,
  // never a command dispatcher or implicit gameplay acceptance.
  const selectedChecks=sorted.map(id=>{
    const test=byId.get(id);
    return Object.freeze({
      id:test.id,required:test.required,given:test.given,
      action:test.action,observe:test.observe,
      expect:Object.freeze({...test.expect}),
      evidenceNeeded:Object.freeze([...test.evidence]),
      status:'NOT_TESTED',
    });
  });
  return Object.freeze({
    schema:1,map:validated.map,revision:validated.revision,
    requestedScope:scope,effectiveScope,reason,
    total:validated.requirements.length,
    required:validated.requirements.filter(r=>r.required).length,
    selected:Object.freeze(sorted),notSelected:validated.requirements.length-sorted.length,
    selectedChecks:Object.freeze(selectedChecks),
    requiredSelected:selectedChecks.filter(x=>x.required).length,
    evidence:Object.freeze(evidence),
    selectionProof:'SOURCE_ONLY_NOT_GAMEPLAY_OR_DIFF_AUTHENTICATION',
    mapAcceptance:'NOT_TESTED',execution:'NOT_EXECUTED'
  });
}
export function compactMapTestPlan(plan){
  if(!plan||plan.schema!==1||!Array.isArray(plan.selected)||
     !Array.isArray(plan.selectedChecks)||
     !Number.isSafeInteger(plan.requiredSelected))refuse();
  return {schema:1,map:plan.map,revision:plan.revision,
    requestedScope:plan.requestedScope,effectiveScope:plan.effectiveScope,
    reason:plan.reason,total:plan.total,required:plan.required,
    selectedCount:plan.selected.length,requiredSelected:plan.requiredSelected,
    requiredNotSelected:plan.required-plan.requiredSelected,
    notSelectedCount:plan.notSelected,notTestedCount:plan.total,
    selectedIds:Object.freeze(plan.selected.slice(0,12)),
    selectedChecks:Object.freeze(plan.selectedChecks.slice(0,12)),
    omittedSelectedIds:Math.max(0,plan.selected.length-12),
    evidenceNeeded:Object.freeze([...plan.evidence]),
    capabilityStatus:Object.freeze(plan.evidence.map(source=>
      Object.freeze({source,state:MAP_QA_CAPABILITY_STATES[source]}))),
    execution:'NOT_EXECUTED',mapAcceptance:'NOT_TESTED',
    foundationProbe:'SEPARATE_TEST_SCOPE'};
}

/**
 * Source-only projection for the EXISTING probe_qa response. It consumes only
 * an already-validated plan and the original Probe's compact result; it never
 * claims map telemetry, gameplay assertions, or a verified QA-world origin.
 *
 * Always bound output and keep independent foundation/map statuses distinct.
 */
export function composeMapQaReport(mapPlan,probeResult,proofGate=null) {
  if(!object(mapPlan)||mapPlan.schema!==1||
     mapPlan.mapAcceptance!=='NOT_TESTED'||
     mapPlan.execution!=='NOT_EXECUTED'||
     typeof mapPlan.map!=='string'||!MAP.test(mapPlan.map)||
     typeof mapPlan.revision!=='string'||!REVISION.test(mapPlan.revision)||
     !Number.isSafeInteger(mapPlan.total)||mapPlan.total<1||
     mapPlan.total>MAX_REQUIREMENTS||
     !Number.isSafeInteger(mapPlan.selectedCount)||
     mapPlan.selectedCount<1||mapPlan.selectedCount>mapPlan.total||
     !Number.isSafeInteger(mapPlan.required)||mapPlan.required<0||
     mapPlan.required>mapPlan.total||
     !list(mapPlan.evidenceNeeded,5,x=>EVIDENCE.has(x))||
     !object(probeResult)||probeResult.schema!==1)refuse();

  const knownStatus=new Set(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW',
    'WAITING','INCOMPLETE','BLOCKED']);
  const status=knownStatus.has(probeResult.status)?
    probeResult.status:'BLOCKED';
  const foundation={status,
    evidenceOrigin:['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW'].includes(status)?
      'UNATTRIBUTED_ORIGINAL_PROBE_LOG':'NO_COMPLETE_AUTHENTICATED_PROBE_RESULT',
    gameplayAcceptance:'NOT_VERIFIED'};
  if(['PROBE_REPORTED_PASS','FAIL','NEEDS_REVIEW'].includes(
      probeResult.probeReportedStatus)) {
    foundation.probeReportedStatus=probeResult.probeReportedStatus;
  }
  if(Number.isSafeInteger(probeResult.expectedCount)&&
      probeResult.expectedCount>=0&&probeResult.expectedCount<=MAX_AUTO_PROBES) {
    foundation.expectedCount=probeResult.expectedCount;
  }
  const counts=probeResult.counts;
  if(object(counts)&&['pass','fail','info','look'].every(key=>
      Number.isSafeInteger(counts[key])&&counts[key]>=0&&
      counts[key]<=MAX_AUTO_PROBES)) {
    foundation.counts=Object.freeze({
      pass:counts.pass,fail:counts.fail,info:counts.info,look:counts.look});
  }
  if(['dev','release'].includes(probeResult.reportedBuild)){
    foundation.reportedBuild=probeResult.reportedBuild;
  }
  // The three runtime proof gates are ALWAYS unsatisfied in this source-only
  // report; a caller cannot remove them using an empty or false proof list.
  const requiredProof=['AUTHORIZED_PLAYER_ADMIN_COMMAND_DELIVERY',
    'EXACT_QA_WORLD_IDENTITY','SOURCE_BOUND_CONTENT_LOG_ORIGIN'];
  const additional=Array.isArray(proofGate?.missingEvidence)?
    proofGate.missingEvidence.filter(code=>typeof code==='string'&&
      /^[A-Z_]{3,56}$/.test(code)): [];
  const missing=[...new Set([...requiredProof,...additional])].slice(0,8);
  return Object.freeze({
    schema:1,kind:'SOURCE_ONLY_QA_REPORT',
    foundation:Object.freeze(foundation),
    map:Object.freeze({
      id:mapPlan.map,revision:mapPlan.revision,
      ...(typeof mapPlan.contractSha256==='string'&&
        /^[a-f0-9]{64}$/.test(mapPlan.contractSha256)?
        {contractSha256:mapPlan.contractSha256}:{}),
      requestedScope:mapPlan.requestedScope,
      effectiveScope:mapPlan.effectiveScope,
      required:mapPlan.required,selected:mapPlan.selectedCount,
      skipped:mapPlan.total-mapPlan.selectedCount,
      tested:0,passed:0,failed:0,notTested:mapPlan.total,
      status:'NOT_TESTED',
    }),
    evidence:Object.freeze({
      mapEvidence:'NOT_COLLECTED',
      needed:Object.freeze([...mapPlan.evidenceNeeded]),
      verifiedEvidenceFamilies:0,
      missingProof:Object.freeze(missing),
    }),
    commandSentByLazyTester:false,gameplayAcceptance:'NOT_VERIFIED',
    acceptance:'NOT_ACCEPTED',
  });
}

let cached=null,ongoing=null;
async function sourceStamp(file){
  if(typeof file!=='string'||!path.isAbsolute(file)||
    path.extname(file).toLowerCase()!=='.json')refuse();
  const stat=await lstat(file,{bigint:true});
  if(!stat.isFile()||stat.isSymbolicLink()||stat.nlink!==1n||
     stat.ino===0n||stat.size<2n||stat.size>BigInt(MAX_BYTES))refuse();
  return [stat.dev,stat.ino,stat.size,stat.mtimeNs,stat.ctimeNs]
    .map(v=>v.toString()).join(':');
}
function digest(bytes){return createHash('sha256').update(bytes).digest('hex');}
export async function mapContractSourceStatus(file,expectedRevision){
  if(typeof expectedRevision!=='string'||!/^[a-f0-9]{64}$/.test(expectedRevision))
    return 'UNAVAILABLE';
  try {
    return digest(Buffer.from(file+'\0'+await sourceStamp(file)))===expectedRevision?
      'UNCHANGED':'CHANGED';
  }catch{return 'UNAVAILABLE';}
}
export async function readMapTestContractCached(file){
  const stamp=await sourceStamp(file);
  const sourceRevision=digest(Buffer.from(file+'\0'+stamp));
  if(cached?.sourceRevision===sourceRevision)
    return {...cached.result,cache:'UNCHANGED_CONTRACT'};
  if(ongoing?.sourceRevision===sourceRevision){
    return {...await ongoing.promise,cache:'UNCHANGED_CONTRACT'};
  }
  cached=null;
  const promise=(async()=>{
    const bytes=await readFile(file);
    if(bytes.length>MAX_BYTES||await sourceStamp(file)!==stamp)refuse();
    let value;
    try{value=JSON.parse(bytes.toString('utf8'));}catch{refuse();}
    const contract=validateMapTestContract(value);
    const result={contract,sourceRevision,contentSha256:digest(bytes)};
    cached={sourceRevision,result};
    return result;
  })();
  ongoing={sourceRevision,promise};
  try {
    return {...await promise,cache:'FRESH_CONTRACT_REVIEW'};
  }finally{
    if(ongoing?.promise===promise)ongoing=null;
  }
}
