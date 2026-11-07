#!/usr/bin/env node
// Optional local read-only CLI. Does not run Graphify/Obsidian/Pele,
// invoke Minecraft, mutate a file, create a cache, or execute a source test.
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { DEFAULT_ROOT, buildKnowledgeIndex, explainResourcePath,
  findResources, graphSummary } from './knowledge-graph.mjs';
import {buildSourceDependencyIndex} from './source-dependencies.mjs';
import {selectSource,sourceImpact,sourceTrace,sourceSummary,sourceGraphProjection,
  sourceDiagram,sourceImpactDiagram,sourceTraceDiagram,sourceImpactMarkdown,
  sourceTraceMarkdown} from './source-reasoning.mjs';

const TICK=String.fromCharCode(96);
function argumentsFor(argv) {
  const result={format:'json',maxHops:4,limit:12};
  for(let i=0;i<argv.length;i++){
    const word=argv[i], needsValue=new Set(['--id','--source-id','--impact','--query','--domain','--format','--max-hops','--limit']);
    if(needsValue.has(word)){
      const value=argv[++i];if(!value||value.startsWith('--'))throw new Error(word+' requires a value');
      result[word.slice(2).replace(/-([a-z])/g,(_,c)=>c.toUpperCase())]=value;continue;
    }
    if(word==='--trace'||word==='--source-trace'){
      const from=argv[++i],to=argv[++i];
      if(!from||!to||from.startsWith('--')||to.startsWith('--'))
        throw new Error(word+' needs two registered IDs or existing source paths');
      result[word==='--trace'?'trace':'sourceTrace']=[from,to];continue;
    }
    if(['--summary','--graph','--audit','--all','--history','--both-directions','--source-summary','--source-graph'].includes(word)){
      result[word.slice(2).replace(/-([a-z])/g,(_,c)=>c.toUpperCase())]=true;continue;
    }
    throw new Error('Unknown argument: '+word);
  }
  result.maxHops=Number(result.maxHops);
  result.limit=Number(result.limit);
  if(!Number.isInteger(result.maxHops)||result.maxHops<1||result.maxHops>6)
    throw new Error('--max-hops must be 1..6');
  if(!Number.isInteger(result.limit)||result.limit<1||result.limit>30)
    throw new Error('--limit must be 1..30');
  if(!['json','markdown','mermaid'].includes(result.format))throw new Error('--format must be json|markdown|mermaid');
  const operations=['summary','graph','audit','id','trace','query',
    'sourceSummary','sourceGraph','sourceId','impact','sourceTrace'].filter(k=>result[k]);
  if(operations.length!==1)throw new Error('Select exactly one documentation or source operation: --summary|--graph|--audit|--id|--trace|--query|--source-summary|--source-graph|--source-id|--impact|--source-trace');
  if(result.domain&&!result.query)throw new Error('--domain only applies to --query');
  if(result.all&&!result.query)throw new Error('--all only applies to --query');
  if(result.bothDirections&&!result.trace&&!result.sourceTrace)
    throw new Error('--both-directions only applies to --trace or --source-trace');
  if(result.history && (result.sourceSummary||result.sourceGraph||result.sourceId||
    result.impact||result.sourceTrace))throw new Error('--history is not a source graph option');
  if(result.format==='mermaid'&&!result.trace&&!result.graph&&!result.impact&&
    !result.sourceTrace&&!result.sourceGraph)
    throw new Error('Mermaid only applies to trace/graph/impact operations');
  if(result.domain&&!result.query)throw new Error('--domain only applies to --query');
  if(result.all&&!result.query)throw new Error('--all only applies to --query');
  return result;
}
export function graphAsMermaid(nodes,edges){
  const selected=new Map(nodes.map((r,i)=>[r.id,'n'+i]));
  const output=['flowchart LR'];
  for(const r of nodes)output.push('  '+selected.get(r.id)+'["'+r.id+'"]');
  for(const edge of edges)if(selected.has(edge.from)&&selected.has(edge.to))
    output.push('  '+selected.get(edge.from)+' -->|'+edge.type+'| '+selected.get(edge.to));
  return output.join('\n');
}
function markdownTrace(trace){
  const out=['# Repository Architecture Trace','','Status: **'+trace.status+'**','',
    'Boundary: '+TICK+trace.proofBoundary+TICK,''];
  if(trace.status==='FOUND'){
    out.push('## Resources','');
    for(const r of trace.nodes){
      out.push('- '+(r.path.endsWith('.md')?'[['+r.path+'|'+r.id+']]':
        TICK+r.id+TICK+' ('+r.path+')')+' — '+r.authority);
    }
    out.push('','## Authored witnesses','');
    for(const s of trace.steps){
      out.push('- '+TICK+s.from+TICK+' -> '+TICK+s.to+TICK+
        ' ('+s.type+', '+s.direction+'): '+TICK+s.evidence.path+':'+s.evidence.line+TICK);
    }
  }else out.push('No confirmed graph path. Unknown edges are not invented.','');
  out.push('','## Mermaid / Pele-compatible diagram','','~~~mermaid',
    graphAsMermaid(trace.nodes,trace.steps),'~~~','');
  return out.join('\n');
}
export function run(argv,root=DEFAULT_ROOT){
  const opt=argumentsFor(argv);
  const sourceRequested=opt.sourceSummary||opt.sourceGraph||opt.sourceId||
    opt.impact||opt.sourceTrace;
  if(sourceRequested){
    // Separate DERIVED code projection; never inject imports into the
    // authoritative documentation navigation graph or native controller.
    const source=buildSourceDependencyIndex(root);
    let result;
    if(opt.sourceSummary)result=sourceSummary(source);
    if(opt.sourceGraph){
      if(opt.format==='mermaid')return sourceDiagram(source.resources,source.edges)+'\n';
      if(opt.format==='markdown')return '# Source declarations only\n\nNo build, runtime, or code-call proof.\n\n~~~mermaid\n'+sourceDiagram(source.resources,source.edges)+'\n~~~\n';
      result=sourceGraphProjection(source);
    }
    if(opt.sourceId){
      const selected=selectSource(source,opt.sourceId)[0];
      result={schemaVersion:1,mode:'SOURCE_FILE_LOOKUP',resource:selected,
        incoming:source.edges.filter(x=>x.to===selected.id),
        outgoing:source.edges.filter(x=>x.from===selected.id),
        unresolved:source.unresolved.filter(x=>x.from===selected.id),
        unparsed:source.unparsed.filter(x=>x.path===selected.path),
        proofBoundary:source.proofBoundary};
    }
    if(opt.impact){
      const impact=sourceImpact(source,opt.impact,{maxHops:opt.maxHops});
      if(opt.format==='mermaid')return sourceImpactDiagram(impact)+'\n';
      if(opt.format==='markdown')return sourceImpactMarkdown(impact);
      result=impact;
    }
    if(opt.sourceTrace){
      const trace=sourceTrace(source,opt.sourceTrace[0],opt.sourceTrace[1],
        {maxHops:opt.maxHops,allowReverse:!!opt.bothDirections});
      if(opt.format==='mermaid')return sourceTraceDiagram(trace)+'\n';
      if(opt.format==='markdown')return sourceTraceMarkdown(trace);
      result=trace;
    }
    return JSON.stringify(result,null,2)+'\n';
  }
  const index=buildKnowledgeIndex(root);
  let output;
  if(opt.summary)output=graphSummary(index);
  if(opt.graph){
    output={schemaVersion:1,mode:'NAVIGATION_GRAPH',
      proofBoundary:index.proofBoundary,
      resources:index.resources,edges:index.edges,
      unresolved:index.unresolved};
    if(opt.format==='mermaid')return graphAsMermaid(index.resources,index.edges)+'\n';
    if(opt.format==='markdown')return '# Repository knowledge navigation\n\nGraph is an extracted view, not a runtime or code call graph.\n\n~~~mermaid\n'+graphAsMermaid(index.resources,index.edges)+'\n~~~\n';
  }
  if(opt.audit){
    const summary=graphSummary(index);
    const missing=index.unresolved.filter(x=>x.reason==='MISSING_OR_UNSAFE');
    output={...summary,mode:'DOCUMENTATION_GRAPH_AUDIT',
      status:missing.length?'NEEDS_REVIEW':'NO_MISSING_OR_UNSAFE_LINKS_DETECTED',
      missingOrUnsafe:missing,
      note:'NOT_CATALOGED references do not assert absent files. No runtime or test execution.'};
  }
  if(opt.id){
    const r=index.byId.get(opt.id);
    if(!r)output={status:'UNKNOWN_RESOURCE',id:opt.id};
    else output={status:'FOUND',resource:r,
      sections:index.resources.filter(x=>x.parent===r.id),
      incoming:index.edges.filter(e=>e.to===r.id),
      outgoing:index.edges.filter(e=>e.from===r.id),
      proofBoundary:index.proofBoundary};
  }
  if(opt.trace){
    const trace=explainResourcePath(index,opt.trace[0],opt.trace[1],
      {maxHops:opt.maxHops,allowReverse:!!opt.bothDirections,includeHistorical:!!opt.history});
    if(opt.format==='mermaid')return graphAsMermaid(trace.nodes,trace.steps)+'\n';
    if(opt.format==='markdown')return markdownTrace(trace);
    output={schemaVersion:1,mode:'REPOSITORY_ARCHITECTURE_TRACE',trace};
  }
  if(opt.query)output={
    mode:'SCOPED_RESOURCE_RETRIEVAL',query:opt.query,domain:opt.domain||'ALL',
    results:findResources(index,opt.query,{domain:opt.domain,all:!!opt.all,
      includeHistorical:!!opt.history,limit:opt.limit}),
    proofBoundary:index.proofBoundary
  };
  return JSON.stringify(output,null,2)+'\n';
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  try{process.stdout.write(run(process.argv.slice(2)));}
  catch(err){console.error('Knowledge navigation: '+err.message);process.exitCode=1;}
}
