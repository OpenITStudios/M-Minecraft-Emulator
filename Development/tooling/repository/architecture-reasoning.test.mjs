// Authored-only canonical source and decision boundary fixtures: NOT_EXECUTED.
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {buildKnowledgeIndex} from './knowledge-graph.mjs';
import {architectureContext,architectureContextMarkdown,
  implementationOwnerWitnesses,canonicalInvariantWitnesses} from './architecture-reasoning.mjs';
import {run} from './knowledge-query.mjs';

const SELECTED='Development/apps/desktop/src/app/bridge/lazyTester.ts';
const IMPORTER='Development/apps/desktop/src/main.ts';
const selectedId='source.file.development-apps-desktop-src-app-bridge-lazytester-ts';
const importerId='source.file.development-apps-desktop-src-main-ts';
function meta(id,domain,role='CONTRACT',authority='CANONICAL',lifecycle='ACTIVE'){
  return ['---','id: '+id,'class: DOCUMENT','domain: '+domain,'role: '+role,
    'authority: '+authority,'lifecycle: '+lifecycle,'---',''].join('\n');
}
function fixture(callback){
  const root=mkdtempSync(join(tmpdir(),'lazytester-architecture-'));
  const put=(path,text)=>{
    const p=join(root,path);mkdirSync(join(p,'..'),{recursive:true});writeFileSync(p,text);
  };
  try{
    put('docs/README.md',meta('document.docs.router','docs','ROUTER')+
      '# Documentation\n\n[System](system/README.md)\n');
    put('docs/system/README.md',meta('document.system.router','system','ROUTER')+
      '# System\n\n[Owners](implementation-map.md)\n[Architecture](architecture.md)\n');
    put('docs/system/implementation-map.md',
      meta('document.system.implementation-map','system','GUIDE')+
      '# Canonical implementation owners\n\nAll desktop paths below start at `Development/apps/desktop/`.\n\n'+
      '| Problem | Existing canonical source | Follow only when needed |\n'+
      '| --- | --- | --- |\n'+
      '| Typed IPC | `src/app/bridge/lazyTester.ts` | Exact current bridge |\n'+
      '| Not an owner | `src/main.ts` | Mentions `src/app/bridge/lazyTester.ts` only |\n');
    put('docs/system/architecture.md',
      meta('document.system.architecture','system')+
      '# Architecture\n\n## Safety and lifecycle invariants\n\n'+
      '1. Original native controller remains the owner.\n'+
      '   Do not create a second state machine.\n'+
      '2. Runtime proof requires real Windows observation.\n\n'+
      '## Historical details\n\n1. These are not current invariant list items.\n');
    put(SELECTED,'export const f=1;\n');
    put(IMPORTER,'import {f} from "./app/bridge/lazyTester.js";\n');
    return callback({root,put});
  }finally{rmSync(root,{recursive:true,force:true});}
}
function fakeSource(){
  const resources=[
    {id:selectedId,path:SELECTED,class:'SOURCE',authority:'CANONICAL'},
    {id:importerId,path:IMPORTER,class:'SOURCE',authority:'CANONICAL'},
  ];
  return {
    resources,byId:new Map(resources.map(r=>[r.id,r])),
    byPath:new Map(resources.map(r=>[r.path,r])),
    edges:[{from:importerId,to:selectedId,type:'DEPENDS_ON',
      basis:'EXTRACTED_SOURCE_DECLARATION',
      sourceSites:[{path:IMPORTER,line:1,column:9,syntax:'STATIC_IMPORT',
        specifier:'./app/bridge/lazyTester.js'}]}],
    contractEdges:[],cargoLocalDependencies:[],cargoUnresolved:[],
    unparsed:[],unresolved:[],coverage:{excludedLanguageFamilies:['RUST_CFG']},
    proofBoundary:'STATIC_DECLARATIONS_NOT_RUNTIME',
  };
}
test('canonical implementation map owner only comes from second cell with source line',()=>fixture(({root})=>{
  const docs=buildKnowledgeIndex(root);
  const owners=implementationOwnerWitnesses(SELECTED,docs);
  assert.equal(owners.status,'AUTHOR_DECLARED_OWNER_REFERENCES');
  assert.equal(owners.matches.length,1);
  assert.equal(owners.matches[0].problem,'Typed IPC');
  assert.equal(owners.matches[0].relation,'EXACT_OWNER_FILE_REFERENCE');
  assert.equal(owners.matches[0].evidence.path,'docs/system/implementation-map.md');
  assert.ok(owners.matches[0].evidence.line>0);
  assert.equal(implementationOwnerWitnesses('Development/apps/desktop/src/missing.ts',
    docs).status,'NO_DIRECT_OWNER_ROW_FOUND_REVIEW_MANUALLY');
}));
test('canonical safety invariants remain exact text witnesses, not auto-applicable facts',()=>fixture(({root})=>{
  const docs=buildKnowledgeIndex(root);
  const constraints=canonicalInvariantWitnesses(docs);
  assert.equal(constraints.witnesses.length,2);
  assert.match(constraints.witnesses[0].text,/second state machine/);
  assert.equal(constraints.witnesses[0].applicability,
    'REVIEW_FOR_SELECTED_SOURCE_NOT_AUTOMATICALLY_INFERRED');
  assert.ok(constraints.witnesses[0].evidence.line>0);
}));
test('architectural context includes impact, source owner, invariants and no automatic PASS',()=>fixture(({root})=>{
  const docs=buildKnowledgeIndex(root);
  const context=architectureContext(fakeSource(),docs,SELECTED);
  assert.equal(context.mode,'ARCHITECTURE_SOURCE_EXPLANATION');
  assert.equal(context.history.status,'NOT_REQUESTED');
  assert.equal(context.decisionReadiness,'MANUAL_SOURCE_REVIEW_REQUIRED');
  assert.ok(context.potentialImpact.impacted.some(x=>x.resource.path===IMPORTER));
  assert.ok(context.obligations.includes('SOURCE_GRAPH_INCOMPLETE_NEVER_ASSUME_NO_IMPACT'));
  assert.match(architectureContextMarkdown(context),/Current canonical invariant text/);
}));
test('no owner or Git read never creates invented authority or decision',()=>fixture(({root,put})=>{
  const docs=buildKnowledgeIndex(root),source=fakeSource();
  docs.byPath.set('docs/system/implementation-map.md',{
    id:'document.system.implementation-map',authority:'REFERENCE',lifecycle:'ACTIVE'});
  const context=architectureContext(source,docs,SELECTED,{history:{
    status:'GIT_READ_UNAVAILABLE',commits:[],
  }});
  assert.equal(context.ownerRouting.matches.length,0);
  assert.ok(context.obligations.includes('DIRECT_CANONICAL_OWNER_ROW_NOT_FOUND'));
  assert.ok(context.obligations.includes('HISTORICAL_GIT_EVIDENCE_INCOMPLETE'));
  assert.equal(context.history.commits.length,0);
}));
test('CLI architecture context remains read-only and preserves unresolved history',()=>fixture(({root})=>{
  const payload=JSON.parse(run(['--architecture-context',SELECTED,'--with-decisions'],
    root,{gitRead:()=>{throw new Error('Git cannot be accessed');}}));
  assert.equal(payload.mode,'ARCHITECTURE_SOURCE_EXPLANATION');
  assert.equal(payload.history.status,'GIT_READ_UNAVAILABLE');
  assert.equal(payload.decisionReadiness,'MANUAL_SOURCE_REVIEW_REQUIRED');
  assert.equal(payload.potentialImpact.status.completeness,'NOT_PROVEN_COMPLETE');
  assert.throws(()=>run(['--with-decisions','--source-summary'],root),
    /--with-decisions requires --architecture-context/);
}));
