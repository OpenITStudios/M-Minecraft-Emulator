// Authored-only negative + positive regressions. Not executed on GitHub.
// No native Minecraft, files outside disposable fixtures or external tools.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdtempSync, mkdirSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {buildSourceDependencyIndex, listOwnedSourceFiles} from './source-dependencies.mjs';
import {selectSource, sourceImpact, sourceTrace, sourceSummary,
  sourceImpactMarkdown, sourceImpactDiagram} from './source-reasoning.mjs';
import {run} from './knowledge-query.mjs';

const requireDesktop=createRequire(new URL('../../apps/desktop/package.json',import.meta.url));
function fixture(fn){
  const root=mkdtempSync(join(tmpdir(),'lazytester-source-'));
  const put=(path,source)=>{
    const dest=join(root,path);
    mkdirSync(join(dest,'..'),{recursive:true});writeFileSync(dest,source);
  };
  try{
    const ui='Development/apps/desktop/src/';
    const rs='Development/apps/desktop/src-tauri/src/';
    put(ui+'lib.ts','export const answer = 42;\n');
    put(ui+'alias.ts','export type Alias = number;\n');
    put(ui+'main.ts',[
      'import {answer} from "./lib.js";',
      'import type {Alias} from "./alias.js";',
      'const x = import("./lazy.js");',
      'const computed = import("./" + "unknown.js");',
      '// import {ghost} from "./ghost.js";',
      'const quote = "import {fake} from \\'./fake.js\\'";',
      'export {answer as copy} from "./lib.js";',
      'void x; void computed; void quote;',
    ].join('\n'));
    put(ui+'lazy.ts','export const lazy = true;\n');
    put(ui+'Screen.svelte',[
      '<script lang="ts">',
      '  import {answer} from "./lib.js";',
      '  export let value: number = answer;',
      '</script>',
      '<div>{value}</div>',
      '<!-- <script> import "./fake.svelte" </script> -->',
    ].join('\n'));
    put(rs+'main.rs',[
      'mod engine;',
      '// mod ghost;',
      'const LITERAL: &str = "mod ghost; use crate::ghost;";',
    ].join('\n'));
    put(rs+'engine/mod.rs',[
      'pub mod first;',
      'pub mod second;',
      '#[path = "relocated.rs"] mod renamed;',
    ].join('\n'));
    put(rs+'engine/first.rs',[
      'use crate::engine::second::Type;',
      'use super::second::Type as ReusedType;',
      '// use crate::engine::ghost;',
      'const CODE: &str = "use crate::engine::phantom;";',
      'pub fn first() {}',
    ].join('\n'));
    put(rs+'engine/second.rs','pub struct Type;\ninclude!("fragment.rs");\n');
    put(rs+'engine/fragment.rs','pub fn source_fragment() {}\n');
    put(rs+'engine/relocated.rs','pub fn relocated() {}\n');
    put('Development/apps/desktop/src-tauri/Cargo.toml',[
      '[dependencies]',
      'virtual-clients-core = { package = "m-bedrock-virtual-clients-core", path = "../../../engine/runtime-core" }',
    ].join('\n'));
    put('Development/engine/runtime-core/Cargo.toml','[package]\nname = "m-bedrock-virtual-clients-core"\n');
    put('Development/engine/runtime-core/src/lib.rs','pub fn core_contract() {}\n');
    put('Development/tooling/agent/runner.mjs','import {answer} from "../../apps/desktop/src/lib.js";\n');
    put('Development/tooling/agent/runner.test.mjs','import "./runner.mjs";\n');
    put('Development/tooling/agent/ignored.cpp','#include "../danger"\n');
    put('Development/tooling/agent/node_modules/phantom.mjs','import "./ghost.mjs";\n');
    return fn({root,put,ui,rs});
  }finally{rmSync(root,{recursive:true,force:true});}
}
const parsers=()=>{
  // Already declared desktop dev dependencies; no install occurs here.
  return {typescript:requireDesktop('typescript'),svelteCompiler:requireDesktop('svelte/compiler')};
};
test('AST TS/Svelte and Rust module/use source witnesses, without comment ghosts',()=>fixture(({root,ui,rs})=>{
  const index=buildSourceDependencyIndex(root,parsers());
  const incoming=index.edges.filter(e=>e.to===index.byPath.get(ui+'lib.ts').id);
  assert.ok(incoming.some(e=>e.sourceSites.some(s=>s.syntax==='STATIC_IMPORT'&&s.line===1)));
  assert.ok(incoming.some(e=>e.sourceSites.some(s=>s.path.endsWith('Screen.svelte')&&s.line===2)));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='IMPORT_TYPE'||
    (s.specifier==='./alias.js'&&s.runtime===false))));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='DYNAMIC_LITERAL')));
  assert.ok(index.unresolved.some(x=>x.reason==='DYNAMIC_COMPUTED'));
  assert.ok(!index.unresolved.some(x=>/ghost|fake|phantom/.test(x.specifier)));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_MOD'&&s.path===rs+'main.rs')));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_MOD'&&s.path===rs+'engine/mod.rs')));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_USE'&&s.path===rs+'engine/first.rs')));
  assert.ok(index.edges.every(e=>e.sourceSites.every(site=>site.path===index.byId.get(e.from).path)),
    'An extracted edge must always retain the importer source path');
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_USE'&&s.path===rs+'engine/first.rs')));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_INCLUDE_LITERAL')));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_MOD'&&
    s.specifier==='renamed')&&e.to===index.byPath.get(rs+'engine/relocated.rs').id));
  assert.equal(index.cargoLocalDependencies.length,1);
  assert.equal(index.cargoLocalDependencies[0].packageAlias,'virtual-clients-core');
  assert.equal(index.edges.some(e=>e.to.includes('ghost')),false);
  assert.equal(index.unparsed.length,0);
}));
test('source impact traverses real importer chain and retains line evidence',()=>fixture(({root,ui})=>{
  const index=buildSourceDependencyIndex(root,parsers());
  const impact=sourceImpact(index,ui+'lib.ts');
  assert.ok(impact.impacted.some(x=>x.resource.path===ui+'main.ts'));
  assert.ok(impact.impacted.some(x=>x.resource.path.endsWith('runner.test.mjs')));
  assert.ok(impact.impacted.some(x=>x.pathToSelected.some(e=>e.sourceSites.some(s=>s.line>0))));
  assert.match(sourceImpactMarkdown(impact),/Potential source change impact/);
  assert.match(sourceImpactDiagram(impact),/flowchart LR/);
  const trace=sourceTrace(index,ui+'main.ts',ui+'lib.ts');
  assert.equal(trace.result,'FOUND');
  assert.ok(trace.steps[0].sourceSites.length>0);
  const reverse=sourceTrace(index,ui+'lib.ts',ui+'main.ts');
  assert.equal(reverse.result,'NO_PATH');
  assert.equal(reverse.status.completeness,'NOT_PROVEN_COMPLETE');
  assert.equal(reverse.status.noObservedPathMeans,'UNKNOWN_NOT_NO_IMPACT');
  const coreImpact=sourceImpact(index,'Development/engine/runtime-core/src/lib.rs');
  assert.equal(coreImpact.potentialCargoDependents.length,1);
  assert.match(sourceImpactMarkdown(coreImpact),/Potential Cargo dependent packages/);
}));
test('missing selections, traversal, aliases and unknown paths fail closed',()=>fixture(({root,ui})=>{
  const index=buildSourceDependencyIndex(root,parsers());
  assert.throws(()=>sourceImpact(index,'Development/apps/desktop/src/nonexistent.ts'),/No existing indexed/);
  assert.throws(()=>sourceImpact(index,'../outside.ts'),/Invalid repository/);
  assert.throws(()=>sourceImpact(index,'Development'),/No existing indexed/);
  assert.throws(()=>sourceTrace(index,ui+'main.ts',ui+'nothing.ts'),/No existing indexed/);
  assert.deepEqual(selectSource(index,ui.slice(0,-1),{directory:true}).length>=4,true);
  assert.throws(()=>sourceImpact(index,ui+'main.ts',{maxHops:7}),/maxHops/);
}));
test('an unavailable TypeScript/Svelte parser yields explicit incomplete status',()=>fixture(({root,ui})=>{
  const index=buildSourceDependencyIndex(root,{typescript:null,svelteCompiler:null});
  assert.equal(index.status,'INCOMPLETE_PARSER_COVERAGE');
  assert.ok(index.unparsed.some(x=>x.reason==='TYPESCRIPT_PARSER_UNAVAILABLE'));
  assert.ok(index.edges.some(e=>e.sourceSites.some(s=>s.syntax==='RUST_MOD')));
  assert.ok(sourceSummary(index).unparsed>0);
  assert.equal(sourceImpact(index,ui+'main.ts').status.status,'PARSER_COVERAGE_INCOMPLETE');
  assert.equal(sourceImpact(index,'Development/apps/desktop/src-tauri/src/main.rs').status.status,
    'GLOBAL_PARSER_COVERAGE_INCOMPLETE');
}));
test('frontend test directory is scanned and its source import becomes a reverse test impact',()=>fixture(({root,put,ui})=>{
  put('Development/apps/desktop/test/regression.test.ts',
    'import {answer} from "../src/lib.js";\n');
  const index=buildSourceDependencyIndex(root,parsers());
  const source=ui+'lib.ts';
  assert.ok(index.resources.some(r=>r.path.endsWith('/test/regression.test.ts')));
  const impact=sourceImpact(index,source);
  assert.ok(impact.impacted.some(item=>item.resource.path.endsWith('/test/regression.test.ts')));
}));
test('ignored storage and installed packages never enter the scanned graph',()=>fixture(({root,put})=>{
  put('Data/Profiles/MC01/session.ts','import "./secret.ts";\n');
  put('App/installed.ts','export const live = true;\n');
  const paths=listOwnedSourceFiles(root);
  assert.ok(paths.every(p=>!p.startsWith('Data/')&&!p.startsWith('App/')&&!p.includes('node_modules')));
  assert.ok(!paths.some(p=>p.endsWith('.cpp')));
}));
test('existing documentation query API stays separate from source mode',()=>fixture(({root})=>{
  assert.match(run(['--source-summary'],root),/SOURCE_FILE_DEPENDENCY_SUMMARY/);
  assert.match(run(['--impact','Development/apps/desktop/src/lib.ts','--format','markdown'],root),/Potential source change impact/);
  assert.throws(()=>run(['--source-summary','--graph'],root),/exactly one/);
  assert.throws(()=>run(['--impact','../Data/secret.ts'],root),/Invalid repository/);
  assert.throws(()=>run(['--source-summary','--history'],root),/not a source graph/);
}));
