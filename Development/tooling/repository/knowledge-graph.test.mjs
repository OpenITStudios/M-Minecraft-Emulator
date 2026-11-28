// Authored-only fixtures. Do not describe as executed without a real test run.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { buildKnowledgeIndex, authoredLinks, explainResourcePath, findResources,
  graphSummary } from './knowledge-graph.mjs';
import {run} from './knowledge-query.mjs';

function metadata(id,domain,role='DOMAIN',authority='CANONICAL',lifecycle='ACTIVE'){
  return ['---','id: '+id,'class: DOCUMENT','domain: '+domain,
    'role: '+role,'authority: '+authority,'lifecycle: '+lifecycle,'---',''].join('\n');
}
function fixture(cb){
  const root=mkdtempSync(join(tmpdir(),'lazytester-knowledge-'));
  const put=(path,content)=>{
    const p=join(root,path);mkdirSync(join(p,'..'),{recursive:true});
    writeFileSync(p,content);
  };
  try{
    put('docs/README.md',metadata('document.docs.router','docs','ROUTER')+
      '# Root\n\n[System](system/README.md)\n');
    put('docs/system/README.md',metadata('document.system.router','system','ROUTER')+
      '# System\n\n[Graph](graph.md)\n');
    put('docs/system/graph.md',metadata('document.system.graph','system','CONTRACT')+
      '# Graph\n\n## Trust boundary\n\n[Rules](../../AGENTS.md)\n');
    put('AGENTS.md','# Agent policy\n');
    return cb({root,put});
  }finally{rmSync(root,{recursive:true,force:true});}
}
test('authored links exclude code fences, code spans, comments and image links',()=>{
  const tick=String.fromCharCode(96),f=tick.repeat(3);
  const input=[
    '[real](system/README.md)',
    f+'markdown',
    '[phantom](system/bad.md)',
    f,
    'text '+tick+'[inline](system/bad2.md)'+tick,
    '<!-- [comment](system/bad3.md) -->',
    '![image](system/bad4.md)',
    '[[docs/system/README.md|System]]',
    '\\[escaped](system/bad5.md)',
    '\\[[docs/system/bad6.md]]',
  ].join('\n');
  const links=[...authoredLinks(input)].map(x=>x.target);
  assert.deepEqual(links,['system/README.md','docs/system/README.md']);
});
test('catalog identity, route trace, section derivation and source provenance',()=>fixture(({root})=>{
  const index=buildKnowledgeIndex(root);
  const trace=explainResourcePath(index,'document.docs.router','document.system.graph');
  assert.equal(trace.status,'FOUND');
  assert.deepEqual(trace.steps.map(s=>s.type),['ROUTES_TO','ROUTES_TO']);
  assert.equal(trace.steps[0].evidence.path,'docs/README.md');
  assert.ok(index.byId.has('document.system.graph.section.trust-boundary'));
  assert.equal(explainResourcePath(index,'document.system.graph','source.rules.agent-policy').status,'FOUND');
  assert.equal(graphSummary(index).unresolvedLinkCount,0);
  assert.ok(findResources(index,'graph',{domain:'system'})
    .some(item=>item.resource.id==='document.system.graph'));
  assert.ok(findResources(index,'trust',{domain:'system'})
    .some(item=>item.matched.some(match=>match.kind==='DOCUMENT_PROSE' &&
      match.path==='docs/system/graph.md' && match.line>0)));
  assert.throws(()=>findResources(index,'graph'),/Choose --domain/);
  assert.match(run(['--trace','document.docs.router','document.system.graph','--format','markdown'],root),/\[\[docs\/system\/graph.md/);
  assert.match(run(['--graph','--format','mermaid'],root),/flowchart LR/);
}));
test('legacy WORKING is not promoted; historical sources remain excluded',()=>fixture(({root,put})=>{
  put('docs/system/graph.md',metadata('document.system.graph','system','REPORT','WORKING')+
    '# Graph\n');
  put('docs/system/old.md',metadata('document.system.old','system','PROCEDURE','WORKING','HISTORICAL')+'# Old\n');
  const index=buildKnowledgeIndex(root);
  assert.equal(index.byId.get('document.system.graph').authority,'REFERENCE');
  assert.equal(index.byId.get('document.system.old').authority,'HISTORICAL');
  assert.equal(explainResourcePath(index,'document.docs.router','document.system.old').status,'UNKNOWN_RESOURCE');
  assert.ok(!findResources(index,'old',{domain:'system'}).some(item=>item.resource.id==='document.system.old'));
  assert.ok(findResources(index,'old',{domain:'system',includeHistorical:true})
    .some(item=>item.resource.id==='document.system.old'));
}));
test('reclassified canonical history cannot route current work even through an active link',()=>fixture(({root,put})=>{
  const id='document.system.retired-provider';
  put('docs/system/README.md',metadata('document.system.router','system','ROUTER')+
    '# System\n\n[Graph](graph.md)\n[Old provider](retired-provider.md)\n');
  put('docs/system/retired-provider.md',metadata(id,'system','CONTRACT','HISTORICAL','HISTORICAL')+
    '# Superseded provider\n\n## Retired route\n');
  const index=buildKnowledgeIndex(root);
  assert.equal(index.byId.get(id).authority,'HISTORICAL');
  assert.equal(index.byId.get(id).lifecycle,'HISTORICAL');
  assert.equal(explainResourcePath(index,'document.system.router',id).status,'UNKNOWN_RESOURCE');
  assert.ok(!index.edges.some(edge=>edge.to===id));
  assert.ok(!findResources(index,'retired-provider',{domain:'system'}).some(item=>item.resource.id===id));
  assert.ok(findResources(index,'retired-provider',{domain:'system',includeHistorical:true})
    .some(item=>item.resource.id===id));
}));
test('SOURCE_ONLY is current source guidance and DRAFT cannot become current authority',()=>fixture(({root,put})=>{
  put('docs/system/graph.md',metadata('document.system.graph','system','GUIDE','WORKING','SOURCE_ONLY')+
    '# Source-only contract\n');
  put('docs/system/draft.md',metadata('document.system.draft','system','SPECIFICATION','WORKING','DRAFT')+
    '# Draft work\n');
  const index=buildKnowledgeIndex(root);
  const sourceOnly=index.byId.get('document.system.graph');
  assert.equal(sourceOnly.lifecycle,'ACTIVE');
  assert.equal(sourceOnly.declaredLifecycle,'SOURCE_ONLY');
  assert.equal(sourceOnly.proofCeiling,'SOURCE_ONLY');
  assert.equal(sourceOnly.authority,'REFERENCE');
  assert.equal(index.byId.get('document.system.draft').authority,'REFERENCE');
  assert.equal(explainResourcePath(index,'document.docs.router','document.system.draft').status,'UNKNOWN_RESOURCE');
}));
test('missing targets cannot create fabricated graph edges',()=>fixture(({root,put})=>{
  put('docs/system/graph.md',metadata('document.system.graph','system')+
    '# Graph\n\n[missing](missing.md)\n');
  const index=buildKnowledgeIndex(root);
  assert.ok(index.unresolved.some(x=>x.reason==='MISSING_OR_UNSAFE'));
  assert.equal(index.edges.some(e=>e.to==='document.system.missing'),false);
  assert.match(run(['--audit'],root),/NEEDS_REVIEW/);
}));
test('duplicate resource IDs fail closed instead of silently replacing owners',()=>fixture(({root,put})=>{
  put('docs/system/clone.md',metadata('document.system.graph','system')+'# Clone\n');
  assert.throws(()=>buildKnowledgeIndex(root),/Duplicate\/invalid Resource ID/);
}));
test('unsupported query/route flags do not broaden silently',()=>fixture(({root})=>{
  assert.throws(()=>run(['--query','policy'],root),/Choose --domain/);
  assert.throws(()=>run(['--summary','--graph'],root),/exactly one/);
  assert.throws(()=>run(['--trace','document.docs.router','x','--max-hops','99'],root),/1\.\.6/);
}));
