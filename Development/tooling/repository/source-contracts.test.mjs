// Authored-only exact-name contract fixtures; deliberately NOT_EXECUTED.
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import {mkdtempSync,mkdirSync,writeFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {buildSourceDependencyIndex,rustTokens} from './source-dependencies.mjs';
import {sourceImpact,sourceTrace} from './source-reasoning.mjs';
import {registeredHandlers,declaredCommands} from './source-contracts.mjs';

const desktop=createRequire(new URL('../../apps/desktop/package.json',import.meta.url));
const FRONT='Development/apps/desktop/src/app/bridge/';
const RUST='Development/apps/desktop/src-tauri/src/';
const BOOT=RUST+'app_bootstrap.rs',COMMANDS=RUST+'commands/native_clients.rs';
function fixture(fn,request='lazy_tester_session'){
  const root=mkdtempSync(join(tmpdir(),'lazytester-tauri-'));
  const put=(path,content)=>{
    const p=join(root,path);mkdirSync(join(p,'..'),{recursive:true});writeFileSync(p,content);
  };
  try {
    put(BOOT,[
      'tauri::generate_handler![commands::native_clients::lazy_tester_session]',
      '// tauri::generate_handler![commands::native_clients::phantom]',
      'const QUOTED: &str = "tauri::generate_handler![commands::native_clients::ghost]";',
      'tauri::generate_handler![commands::native_clients::lazy_tester_session]',
    ].join('\n'));
    put(COMMANDS,[
      '#[tauri::command]',
      'pub async fn lazy_tester_session() {}',
      '// #[tauri::command] pub fn phantom() {}',
    ].join('\n'));
    put(FRONT+'invokeRuntime.ts',
      'export async function invokeRuntime<T>(cmd: string):Promise<T>{return undefined as T;}\n');
    put(FRONT+'lazyTester.ts',[
      'import {invokeRuntime} from "./invokeRuntime.js";',
      'const value=invokeRuntime<unknown>("'+request+'");',
      'const quoted="invokeRuntime(ghost)";',
      'void value; void quoted;',
    ].join('\n'));
    return fn({root,put});
  }finally {rmSync(root,{recursive:true,force:true});}
}
function parsers(){return {typescript:desktop('typescript'),svelteCompiler:desktop('svelte/compiler')}}
test('Tauri registration + backend command + frontend literal all match with witnesses',()=>fixture(({root})=>{
  const index=buildSourceDependencyIndex(root,parsers());
  assert.equal(index.tauri.bindings.length,1);
  assert.equal(index.tauri.handlerGroups,2);
  assert.equal(index.tauri.bindings[0].registrationGroupCount,2);
  assert.equal(index.tauri.gaps.length,0);
  assert.equal(index.contractEdges.length,2);
  assert.ok(index.contractEdges.every(e=>e.sourceSites[0].path===FRONT+'lazyTester.ts'));
  assert.ok(index.tauri.bindings[0].witnesses.registered.every(x=>x.path===BOOT));
  const impact=sourceImpact(index,COMMANDS);
  assert.ok(impact.impacted.some(i=>i.resource.path===FRONT+'lazyTester.ts'));
  const trace=sourceTrace(index,FRONT+'lazyTester.ts',COMMANDS);
  assert.equal(trace.result,'FOUND');
  assert.equal(trace.steps[0].type,'TAURI_COMMAND_NAME_BINDING');
}));
test('missing registered frontend command becomes unknown, not invented edge',()=>fixture(({root})=>{
  const index=buildSourceDependencyIndex(root,parsers());
  assert.equal(index.tauri.bindings.length,0);
  assert.ok(index.tauri.gaps.some(g=>g.reason==='FRONTEND_COMMAND_NOT_REGISTERED'));
  assert.equal(sourceImpact(index,COMMANDS).status.status,'GLOBAL_UNRESOLVED_REFERENCES_REMAIN');
},'not_registered'));
test('Rust tokenizer excludes fake registrations in comments and strings',()=>{
  const source=[
    '/* tauri::generate_handler![commands::native_clients::ghost] */',
    'tauri::generate_handler![commands::native_clients::real]',
  ].join('\n');
  const parsed=registeredHandlers(source,rustTokens);
  assert.deepEqual(parsed.result.map(x=>x.command),['real']);
  const definitions=declaredCommands([
    '// #[tauri::command] pub fn ghost() {}',
    '#[tauri::command]',
    'pub async fn real() {}',
  ].join('\n'),rustTokens);
  assert.deepEqual(definitions.result.map(x=>x.command),['real']);
});
test('missing frontend parser is declared as a knowledge gap',()=>fixture(({root})=>{
  const index=buildSourceDependencyIndex(root,{typescript:null,svelteCompiler:null});
  assert.equal(index.tauri.bindings.length,0);
  assert.ok(index.tauri.gaps.some(g=>g.reason==='TYPESCRIPT_PARSER_UNAVAILABLE'));
  assert.equal(sourceImpact(index,COMMANDS).status.completeness,'NOT_PROVEN_COMPLETE');
}));
