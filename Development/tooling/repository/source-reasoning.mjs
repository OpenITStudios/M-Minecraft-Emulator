import {SOURCE_DIRS} from './source-dependencies.mjs';
// Conservative SOURCE_GRAPH reasoning only; no execution or source writes.
// Stable file IDs and exact paths are derived from active source, not another
// owner registry. Consult docs/system/implementation-map.md for semantics.
export function selectSource(index,input,{directory=false}={}){
  if(typeof input!=='string'||!input||input.includes('\0')||
    input.includes('\\')||input.startsWith('/')||/^[A-Za-z]:/.test(input)||
    input.includes('://')||input.split('/').includes('..')||
    input.split('/').includes('.'))throw new Error('Invalid repository-relative SOURCE selection');
  const byId=index.byId.get(input);
  if(byId)return [byId];
  const path=input.replace(/\/+$/,'');
  if(index.byPath.has(path))return [index.byPath.get(path)];
  if(directory && SOURCE_DIRS.some(root=>path===root||path.startsWith(root+'/'))){
    const matches=index.resources.filter(r=>r.path.startsWith(path+'/'));
    if(matches.length)return matches;
  }
  throw new Error('No existing indexed source file or registered directory: '+input);
}
const adjacency=(edges,key)=>{
  const map=new Map();
  for(const edge of edges){
    if(!map.has(edge[key]))map.set(edge[key],[]);
    map.get(edge[key]).push(edge);
  }
  return map;
};
function graphStatus(index, relevantIds){
  const unknown=index.unparsed.filter(item=>relevantIds.has(index.byPath.get(item.path)?.id));
  const unresolved=index.unresolved.filter(item=>relevantIds.has(item.from));
  return {
    confidence:'POTENTIAL_STATIC_FILE_REACHABILITY_ONLY',
    status:unknown.length?'PARSER_COVERAGE_INCOMPLETE':
      unresolved.length?'UNRESOLVED_REFERENCES_REMAIN':'SOURCE_DECLARATIONS_OBSERVED',
    parserGaps:unknown,unresolved,
    proofBoundary:index.proofBoundary,
  };
}
export function sourceImpact(index,input,{maxHops=4}={}){
  if(!Number.isInteger(maxHops)||maxHops<1||maxHops>6)throw new Error('maxHops must be 1..6');
  const selected=selectSource(index,input,{directory:true});
  const originals=new Set(selected.map(r=>r.id));
  const visited=new Set(originals);
  const incoming=adjacency(index.edges,'to');
  const queue=selected.map(resource=>({resource,distance:0,pathToSelected:[]}));
  const impacted=[];
  let depthTruncated=false;
  for(let i=0;i<queue.length;i++){
    const current=queue[i];
    for(const edge of incoming.get(current.resource.id)||[]){
      if(visited.has(edge.from))continue;
      if(current.distance>=maxHops){depthTruncated=true;continue;}
      visited.add(edge.from);
      const resource=index.byId.get(edge.from);
      const result={resource,distance:current.distance+1,
        pathToSelected:[edge,...current.pathToSelected]};
      queue.push(result);
      impacted.push(result);
    }
  }
  impacted.sort((a,b)=>a.distance-b.distance||a.resource.path.localeCompare(b.resource.path));
  const relevantIds=new Set([...originals,...impacted.map(item=>item.resource.id)]);
  const selectedPaths=selected.map(r=>r.path);
  const potentialCargoDependents=(index.cargoLocalDependencies||[]).filter(dep=>
    selectedPaths.some(p=>p.startsWith(dep.toManifest.slice(0,-'Cargo.toml'.length))))
    .map(dep=>({...dep,confidence:'POTENTIAL_PACKAGE_IMPACT_NOT_PER_FILE_DEPENDENCY'}));
  return {
    schemaVersion:1,mode:'SOURCE_FILE_IMPACT',
    requestedPathOrId:input,selected,
    status:graphStatus(index,relevantIds),
    depth:{maxHops,depthTruncated},
    impacted,potentialCargoDependents,
    note:'Reverse reachability from literal source imports/mod/use declarations is potential affected-file evidence, NOT a guarantee of changed symbols, successful compilation, test failure or runtime behavior.',
  };
}
export function sourceTrace(index,from,to,{maxHops=4,allowReverse=false}={}){
  if(!Number.isInteger(maxHops)||maxHops<1||maxHops>6)throw new Error('maxHops must be 1..6');
  const start=selectSource(index,from),goal=selectSource(index,to);
  const target=goal[0].id,forward=adjacency(index.edges,'from'),
    backward=allowReverse?adjacency(index.edges,'to'):new Map();
  const queue=[{id:start[0].id,ids:[start[0].id],steps:[]}];
  const visited=new Set([start[0].id]);
  for(let i=0;i<queue.length;i++){
    const current=queue[i];
    if(current.id===target){
      const relevant=new Set(current.ids);
      return {schemaVersion:1,mode:'SOURCE_FILE_TRACE',result:'FOUND',
        from:start[0],to:goal[0],nodes:current.ids.map(id=>index.byId.get(id)),
        steps:current.steps,scope:{maxHops,allowReverse},
        status:graphStatus(index,relevant)};
    }
    if(current.steps.length>=maxHops)continue;
    const edges=[
      ...(forward.get(current.id)||[]).map(edge=>({edge,next:edge.to,direction:'FORWARD'})),
      ...(backward.get(current.id)||[]).map(edge=>({edge,next:edge.from,direction:'REVERSE'})),
    ];
    for(const item of edges){
      if(visited.has(item.next))continue;
      visited.add(item.next);
      queue.push({id:item.next,ids:[...current.ids,item.next],
        steps:[...current.steps,{...item.edge,direction:item.direction}]});
    }
  }
  return {schemaVersion:1,mode:'SOURCE_FILE_TRACE',result:'NO_PATH',
    from:start[0],to:goal[0],nodes:[],steps:[],
    scope:{maxHops,allowReverse},status:graphStatus(index,new Set([start[0].id,goal[0].id])),
    note:'NO_PATH means no observed path in the scanned declarations, NOT independence.'};
}
export function sourceSummary(index){
  const counts=(items,key)=>Object.fromEntries([...new Set(items.map(key))].sort()
    .map(name=>[name,items.filter(x=>key(x)===name).length]));
  return {
    schemaVersion:1,mode:'SOURCE_FILE_DEPENDENCY_SUMMARY',
    status:index.status,proofBoundary:index.proofBoundary,
    resourceCount:index.resources.length,
    cargoLocalDependencies:index.cargoLocalDependencies||[],
    unresolvedCargoPaths:index.cargoUnresolved||[],
    filesByDomain:counts(index.resources,r=>r.domain),
    sourceEdges:index.edges.length,
    declarationSites:index.siteCount,
    externalOrAliasReferencesSkipped:index.externalCount,
    nonCodeResourcesSkipped:index.resourceCount,
    missingOrUnresolved:index.unresolved.length,
    unparsed:index.unparsed.length,
    parserAvailability:index.parserAvailability,
    diagnostics:{
      unparsed:index.unparsed.slice(0,30),
      unresolved:index.unresolved.slice(0,30),
    },
    excludedFromAnalysis:['C++/C','PowerShell','Cargo package/dependency resolution',
      'macro-expanded Rust','conditional cfg/feature reachability',
      'computed dynamic imports','TS path aliases','runtime/Minecraft execution'],
  };
}
export function sourceGraphProjection(index){
  return {schemaVersion:1,mode:index.mode,status:index.status,
    proofBoundary:index.proofBoundary,parserAvailability:index.parserAvailability,
    resources:index.resources,edges:index.edges,unresolved:index.unresolved,unparsed:index.unparsed,
    cargoLocalDependencies:index.cargoLocalDependencies||[],
    cargoUnresolved:index.cargoUnresolved||[]};
}
export function sourceDiagram(resources,edges){
  const unique=new Map(resources.map(r=>[r.id,r]));
  const ids=[...unique.keys()].sort();
  const alias=new Map(ids.map((id,i)=>[id,'n'+i]));
  const lines=['flowchart LR'];
  for(const id of ids){
    lines.push('  '+alias.get(id)+'['+JSON.stringify(unique.get(id).path)+']');
  }
  const seen=new Set();
  for(const e of edges){
    if(!alias.has(e.from)||!alias.has(e.to))continue;
    const key=e.from+'|'+e.to;
    if(seen.has(key))continue;
    seen.add(key);
    lines.push('  '+alias.get(e.from)+' -->|declares dependency| '+alias.get(e.to));
  }
  return lines.join('\n');
}
export function sourceImpactDiagram(impact){
  const resources=[...impact.selected,...impact.impacted.map(i=>i.resource)];
  const edges=impact.impacted.flatMap(item=>item.pathToSelected);
  return sourceDiagram(resources,edges);
}
export function sourceTraceDiagram(trace){
  return sourceDiagram(trace.nodes,trace.steps);
}
export function sourceImpactMarkdown(impact){
  const tick=String.fromCharCode(96);
  const lines=['# Potential source change impact','','Requested: '+tick+impact.requestedPathOrId+tick,
    'Result: **'+impact.status.status+'**','Proof: '+tick+impact.status.proofBoundary+tick,
    'Depth: '+impact.depth.maxHops+' (truncated: '+impact.depth.depthTruncated+')','','## Selected source',''];
  for(const r of impact.selected)lines.push('- [['+r.path+'|'+r.id+']]');
  lines.push('','## Potentially affected files','','An import/module edge does not establish code execution, affected symbols, or a regression.','');
  for(const i of impact.impacted){
    lines.push('- [['+i.resource.path+'|'+i.resource.id+']] — '+i.distance+' hop(s)'+
      (i.resource.kind==='AUTHORED_TEST_NOT_EXECUTION_PROOF'?' [authored test]':''));
    for(const e of i.pathToSelected){
      for(const site of e.sourceSites.slice(0,4))lines.push('  - '+tick+
        site.path+':'+site.line+':'+site.column+tick+' '+site.syntax+
        ' '+tick+site.specifier+tick);
      if(e.sourceSites.length>4)lines.push('  - '+(e.sourceSites.length-4)+' additional sites in JSON');
    }
  }
  if(!impact.impacted.length)lines.push('No reverse dependency found within the scan; this is NOT proof of no impact.');
  lines.push('','## Potential Cargo dependent packages (not per-file edges)','');
  if(!impact.potentialCargoDependents.length)
    lines.push('No directly affected local Cargo package declaration identified.');
  for(const dep of impact.potentialCargoDependents){
    lines.push('- '+tick+dep.fromManifest+tick+' declares dependency on '+
      tick+dep.toManifest+tick+' at '+tick+dep.evidence.path+':'+
      dep.evidence.line+tick+'. This is a possible crate-level impact, not a code call.');
  }
  const relevant=[...impact.status.parserGaps,...impact.status.unresolved];
  lines.push('','## Unresolved coverage and references','');
  if(!relevant.length)lines.push('No parser gaps or unresolved declarations among indexed reachable files.');
  for(const gap of relevant.slice(0,30)){
    lines.push('- '+tick+gap.path+(gap.line?':'+gap.line:'')+tick+' '+
      (gap.reason||gap.specifier||'UNKNOWN'));
  }
  if(relevant.length>30)lines.push('- '+(relevant.length-30)+' additional unresolved sites in JSON');
  lines.push('','## Mermaid / Pele-compatible affected-file view','','~~~mermaid',
    sourceImpactDiagram(impact),'~~~','');
  return lines.join('\n');
}
export function sourceTraceMarkdown(trace){
  const tick=String.fromCharCode(96);
  const lines=['# Source declaration trace','','Result: **'+trace.result+'**',
    'Coverage: '+trace.status.status,
    'Proof: '+tick+trace.status.proofBoundary+tick,'',
    'From [['+trace.from.path+'|'+trace.from.id+']] to [['+trace.to.path+'|'+trace.to.id+']]',''];
  if(trace.result==='FOUND'){
    lines.push('## Declared source path','');
    for(const e of trace.steps){
      lines.push('- '+tick+e.from+tick+' -> '+tick+e.to+tick+' ('+e.direction+')');
      for(const site of e.sourceSites.slice(0,5))
        lines.push('  - '+tick+site.path+':'+site.line+':'+site.column+tick+
          ' '+site.syntax+' '+tick+site.specifier+tick);
    }
  }else lines.push('No verified source-declaration path within scan; do not infer independence.');
  lines.push('','## Mermaid / Pele-compatible trace','','~~~mermaid',
    sourceTraceDiagram(trace),'~~~','');
  return lines.join('\n');
}
