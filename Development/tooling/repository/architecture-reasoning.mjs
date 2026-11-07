// Bounded architecture explanation: existing canonical owner rows,
// current source change impact, and optional historical Git witnesses.
// Does NOT create a new decision store, authority graph or execution agent.
import {selectSource,sourceImpact} from './source-reasoning.mjs';

const IMPLEMENTATION_MAP='docs/system/implementation-map.md';
const ARCHITECTURE='docs/system/architecture.md';
const DESKTOP_BASE='Development/apps/desktop/';
const SOURCE_ROOTS=['Development/','.agents/'];

export const REASONING_CEILING=
  'AUTHORED_OWNER_AND_INVARIANT_TEXT_PLUS_POTENTIAL_SOURCE_IMPACT_NOT_RUNTIME_OR_SAFETY_PROOF';

function relativeOwnerToken(token){
  if(typeof token!=='string')return undefined;
  const value=token.trim().replace(/^\.\//,'');
  if(!value||value.includes('\\')||value.includes('\0')||
    value.startsWith('/')||value.split('/').includes('..'))return undefined;
  if(SOURCE_ROOTS.some(prefix=>value.startsWith(prefix)))return value;
  if(value.startsWith('src/')||value.startsWith('src-tauri/'))
    return DESKTOP_BASE+value;
  return undefined; // No guessing for bare module names or aliases.
}
function ownerMatch(sourcePath,token){
  const resolved=relativeOwnerToken(token);
  if(!resolved)return undefined;
  if(sourcePath===resolved)return {relation:'EXACT_OWNER_FILE_REFERENCE',resolved};
  if(resolved.endsWith('/')&&sourcePath.startsWith(resolved))
    return {relation:'EXPLICIT_OWNER_DIRECTORY_REFERENCE',resolved};
  // A no-extension prefix is NOT evidence of directory ownership:
  // there may be both name.rs and name/ or a retired prefix.
  return undefined;
}
export function implementationOwnerWitnesses(sourcePath,knowledgeIndex,{limit=12}={}){
  const resource=knowledgeIndex.byPath?.get(IMPLEMENTATION_MAP);
  const content=knowledgeIndex.contents?.get(IMPLEMENTATION_MAP);
  if(!resource||resource.lifecycle!=='ACTIVE'||resource.authority!=='CANONICAL'||
    typeof content!=='string')return {
      status:'CANONICAL_IMPLEMENTATION_MAP_UNAVAILABLE',matches:[]};
  const matches=[],lines=content.split(/\r?\n/);
  for(let i=0;i<lines.length;i++){
    const row=lines[i];
    if(!row.startsWith('|')||row.startsWith('| ---'))continue;
    const cells=row.split('|').slice(1,-1).map(c=>c.trim());
    if(cells.length<3||cells[0]==='Problem')continue;
    // Only the authored "Existing canonical source" cell claims owner
    // routing; mentions in the explanatory third cell are not owners.
    for(const m of cells[1].matchAll(/`([^`\r\n]+)`/g)){
      const found=ownerMatch(sourcePath,m[1]);
      if(!found)continue;
      matches.push({problem:cells[0].slice(0,180),source:sourcePath,
        authoredToken:m[1],...found,
        authority:'CANONICAL_IMPLEMENTATION_MAP_ROW_NOT_CODE_PROOF',
        evidence:{path:IMPLEMENTATION_MAP,documentId:resource.id,
          line:i+1,target:m[1]}});
    }
  }
  matches.sort((a,b)=>
    Number(b.relation==='EXACT_OWNER_FILE_REFERENCE')-
      Number(a.relation==='EXACT_OWNER_FILE_REFERENCE')||
    a.evidence.line-b.evidence.line);
  return {status:matches.length?'AUTHOR_DECLARED_OWNER_REFERENCES':
    'NO_DIRECT_OWNER_ROW_FOUND_REVIEW_MANUALLY',
    matches:matches.slice(0,Math.min(limit,12)),
    omitted:Math.max(0,matches.length-Math.min(limit,12))};
}
export function canonicalInvariantWitnesses(knowledgeIndex,{limit=8}={}){
  const resource=knowledgeIndex.byPath?.get(ARCHITECTURE);
  const content=knowledgeIndex.contents?.get(ARCHITECTURE);
  if(!resource||resource.lifecycle!=='ACTIVE'||resource.authority!=='CANONICAL'||
    typeof content!=='string')return {
      status:'CANONICAL_ARCHITECTURE_UNAVAILABLE',witnesses:[]};
  const lines=content.split(/\r?\n/);
  const heading=lines.findIndex(line=>/^## Safety and lifecycle invariants\s*$/.test(line));
  if(heading<0)return {status:'AUTHOR_SECTION_MISSING',witnesses:[]};
  const witnesses=[];
  let current;
  function save(){
    if(!current)return;
    const {parts,...entry}=current;
    witnesses.push({...entry,text:parts.join(' ').replace(/\s+/g,' ').slice(0,900),
      applicability:'REVIEW_FOR_SELECTED_SOURCE_NOT_AUTOMATICALLY_INFERRED'});
  }
  for(let i=heading+1;i<lines.length;i++){
    const line=lines[i];
    if(/^##\s/.test(line))break;
    const numbered=line.match(/^(\d+)\.\s+(.+)$/);
    if(numbered){
      save();
      current={key:'ARCHITECTURE_LIST_ITEM_'+numbered[1],
        evidence:{path:ARCHITECTURE,documentId:resource.id,
          heading:'Safety and lifecycle invariants',line:i+1},
        parts:[numbered[2].trim()]};
    }else if(current&&line.trim()&&!/^(?:---|```|~~~)/.test(line)){
      current.parts.push(line.trim());
    }
  }
  save();
  return {status:'AUTHORED_CANONICAL_TEXT_REQUIRES_APPLICABILITY_REVIEW',
    witnesses:witnesses.slice(0,Math.min(limit,8)),
    totalInSection:witnesses.length};
}
export function architectureContext(sourceIndex,knowledgeIndex,selection,{
  maxHops=4,limit=12,history,
}={}){
  const selected=selectSource(sourceIndex,selection)[0];
  const impact=sourceImpact(sourceIndex,selected.path,{maxHops});
  const owners=implementationOwnerWitnesses(selected.path,knowledgeIndex,{limit});
  const constraints=canonicalInvariantWitnesses(knowledgeIndex);
  const obligations=['VERIFY_CURRENT_SOURCE_SEMANTIC_OWNER',
    'CHECK_DIRECT_CALLERS_AND_AUTHORED_TESTS',
    'REVIEW_INVARIANT_APPLICABILITY',
    'DO_NOT_PROMOTE_GIT_TRAILER_OR_STATIC_DECLARATION_TO_RUNTIME_PASS'];
  if(owners.matches.length===0)obligations.push('DIRECT_CANONICAL_OWNER_ROW_NOT_FOUND');
  if(impact.status.completeness!=='PROVEN_COMPLETE')
    obligations.push('SOURCE_GRAPH_INCOMPLETE_NEVER_ASSUME_NO_IMPACT');
  if(history&&history.status!=='HISTORY_BOUNDED')
    obligations.push('HISTORICAL_GIT_EVIDENCE_INCOMPLETE');
  return {
    schemaVersion:1,mode:'ARCHITECTURE_SOURCE_EXPLANATION',
    selected,ownerRouting:owners,canonicalInvariants:constraints,
    potentialImpact:impact,
    history:history??{status:'NOT_REQUESTED',
      note:'Git history remains opt-in; there is no implicit history scan.'},
    decisionReadiness:'MANUAL_SOURCE_REVIEW_REQUIRED',
    proofBoundary:REASONING_CEILING,
    obligations,
    note:'Owner rows are authored navigation, constraints are active text, imports are potential impact, and Git trailers are historical claims. None authorizes mutation or proves Windows/Minecraft behavior.',
  };
}
export function architectureContextMarkdown(context){
  const tick=String.fromCharCode(96);
  const lines=['# Architecture source context','',
    'Source: '+tick+context.selected.path+tick,
    'Readiness: **'+context.decisionReadiness+'**',
    'Proof ceiling: '+tick+context.proofBoundary+tick,'',
    '## Authored owner references','',
    'Status: '+context.ownerRouting.status,''];
  if(context.ownerRouting.matches.length===0)
    lines.push('No exact canonical owner row matched this source. Review the live implementation map and source manually.','');
  for(const o of context.ownerRouting.matches)
    lines.push('- '+o.problem+' — '+o.relation+' via '+tick+o.authoredToken+
      tick+' at '+tick+o.evidence.path+':'+o.evidence.line+tick);
  lines.push('','## Current canonical invariant text (applicability not inferred)','');
  for(const invariant of context.canonicalInvariants.witnesses)
    lines.push('- '+invariant.key+': '+invariant.text+
      ' ('+tick+invariant.evidence.path+':'+invariant.evidence.line+tick+')');
  lines.push('','## Potential reverse source impact','',
    'Coverage: **'+context.potentialImpact.status.status+
    '**; completeness: **'+context.potentialImpact.status.completeness+'**',
    'Global parser gaps: '+context.potentialImpact.status.globalParserGapCount+
    '; global unresolved: '+context.potentialImpact.status.globalUnresolvedCount,
    'Potential impacted files: '+context.potentialImpact.impacted.length,'');
  for(const item of context.potentialImpact.impacted.slice(0,20)){
    lines.push('- '+tick+item.resource.path+tick+' ('+item.distance+' hop(s))');
    const edge=item.pathToSelected[0];
    const site=edge?.sourceSites?.[0];
    if(site)lines.push('  - witness '+tick+site.path+':'+site.line+':'+site.column+
      tick+' ('+site.syntax+')');
  }
  if(!context.potentialImpact.impacted.length)
    lines.push('No importers observed within scope; this is NOT proof of independence.');
  lines.push('','## Historical decisions','');
  if(context.history.status==='NOT_REQUESTED')lines.push('NOT_REQUESTED (opt in with --with-decisions).');
  else {
    lines.push('Status: '+context.history.status,'');
    for(const entry of context.history.commits??[]){
      lines.push('- '+tick+entry.sha+tick+' Work '+tick+entry.work+
        tick+'; state '+tick+entry.state+tick+
        '; recorded decision: '+(entry.decision??'NOT_RECORDED')+
        ' [HISTORICAL_AUTHOR_CLAIM_ONLY]');
    }
    if(!context.history.commits?.length)
      lines.push('No directly witnessed decision in the bounded available history; do not infer none exists.');
  }
  lines.push('','## Source review obligations','');
  for(const item of context.obligations)lines.push('- '+tick+item+tick);
  return lines.join('\n')+'\n';
}
