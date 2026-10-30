import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname,resolve} from 'node:path';
import {validatedRegistry,classifyPath,planForPaths,legacyArgs,parseFlags} from './controller.mjs';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const registry=JSON.parse(readFileSync(new URL('./tasks.json',import.meta.url),'utf8'));
const tasks=validatedRegistry(registry);
const SHA='a'.repeat(40);

test('catalog defines unique bounded fixed tasks and never exposes shell executors',()=>{
  assert.ok(tasks.length>=12);
  assert.equal(new Set(tasks.map(t=>t.id)).size,tasks.length);
  assert.deepEqual(tasks.filter(t=>t.requiresApproval).map(t=>t.id).sort(),
    ['build.candidate','cargo.lock','environment.setup','source.preflight'].sort());
  assert.equal(tasks.find(t=>t.id==='storage.plan').risk,'read');
  for(const task of tasks) assert.ok(['node','pwsh'].includes(legacyArgs(task).type));
  assert.ok(!tasks.some(t=>t.command==='g0'));
});

test('registry refuses injected command, duplicate identity, and unreviewed mode',()=>{
  const modified=structuredClone(registry);
  modified.tasks[0].command='powershell -EncodedCommand';
  assert.throws(()=>validatedRegistry(modified),/Invalid/);
  const duplicate=structuredClone(registry);
  duplicate.tasks[1].id=duplicate.tasks[0].id;
  assert.throws(()=>validatedRegistry(duplicate),/duplicate/);
  const mode=structuredClone(registry);
  mode.tasks[0].mode='report';
  assert.throws(()=>validatedRegistry(mode),/mode/);
  const rogue=structuredClone(registry);
  rogue.tasks[0].script='../../Data/worlds/secret';
  assert.throws(()=>validatedRegistry(rogue),/Invalid/);
});

test('source routing uses minimum relevant checks, but unknown and mixed fail wide',()=>{
  assert.deepEqual(planForPaths([],SHA).taskIds,[]);
  assert.deepEqual(planForPaths(['docs/system/architecture.md'],SHA).taskIds,['repository.validate']);
  assert.deepEqual(planForPaths(['Development/apps/desktop/src/LazyTesterApp.svelte'],SHA).taskIds,['frontend.validate']);
  assert.deepEqual(planForPaths(['Development/engine/runtime-core/src/lib.rs'],SHA).taskIds,['rust.validate']);
  assert.deepEqual(planForPaths(['Development/tooling/acceptance/windows/bootstrap-ordering/compat.cpp'],SHA).taskIds,['source.validate']);
  assert.deepEqual(planForPaths(['Development/apps/desktop/src/a.ts','Development/engine/runtime-core/src/lib.rs'],SHA).taskIds,['source.validate']);
  assert.deepEqual(planForPaths(['Development/toolchain.json'],SHA).taskIds,['source.validate']);
});

test('protected or suspicious paths never produce executable routing',()=>{
  for(const path of ['Data/Launchers/Minecraft-1/world','workspace/private/a','App/setup.exe','.git/config']) {
    assert.equal(classifyPath(path),'protected');
    assert.equal(planForPaths([path],SHA).status,'BLOCKED');
  }
  for(const path of ['../outside','/tmp/input','foo\\bar','-bad']) {
    assert.equal(classifyPath(path),'unknown');
    assert.deepEqual(planForPaths([path],SHA).taskIds,['source.validate']);
  }
});

test('no arbitrary argument vectors can reach a shell command',()=>{
  for(const task of tasks) {
    const command=legacyArgs(task);
    if(command.type==='pwsh') {
      assert.equal(command.args.slice(0,4).join(' '),'-NoLogo -NoProfile -NonInteractive -File');
      assert.ok(command.args.at(-1)==='report'||command.args.at(-1)==='plan'||
        command.args.at(-1)==='cargo'||command.args.at(-1)==='explain'||command.args.length===6);
    }
  }
  assert.throws(()=>legacyArgs({command:'evil','id':'bad'}),/Unregistered/);
});

test('AI controller CLI arguments are constrained and source-pinned',()=>{
  assert.equal(parseFlags(['--json'],'catalog').json,true);
  assert.equal(parseFlags(['--json','--paths','docs/README.md'],'plan').paths[0],'docs/README.md');
  assert.equal(parseFlags(['frontend.validate','--expect-sha',SHA],'run').expectSha,SHA);
  assert.equal(parseFlags(['build.candidate','--expect-sha',SHA,'--approve','build.candidate'],'run').approval,'build.candidate');
  assert.throws(()=>parseFlags(['--approve','build.candidate'],'plan'),/Unexpected/);
  assert.throws(()=>parseFlags(['frontend.validate','--bogus'],'run'),/Unexpected/);
  assert.throws(()=>parseFlags(['frontend.validate','--json','--json'],'run'),/Unexpected/);
});

test('existing source verifier and developer executor remain the single owners',()=>{
  const launcher=readFileSync(resolve(root,'DEV.cmd'),'utf8');
  const local=readFileSync(resolve(root,'Development/tooling/windows-toolchain/dev.ps1'),'utf8');
  assert.match(launcher,/if \/I "%~1"=="bootstrap"/);
  assert.match(launcher,/tooling\\developer\\controller\.mjs/);
  assert.match(launcher,/"%LOCAL_PWSH%" -NoLogo -NoProfile/);
  assert.match(local,/Set-LocalDeveloperEnvironment -Root \$Root/);
  assert.match(local,/"package" \{ Invoke-RepositoryNpm/);
  assert.match(local,/"storage"/);
});
