import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname,resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {validatedRegistry,classifyPath,planForPaths,legacyArgs,parseFlags,isPinnedNodeProcess,classifyCompletion,runOwnedChild,withBuildLock} from './controller.mjs';

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

test('pinned Node admission verifies exact executable path, not only version',()=>{
  assert.equal(isPinnedNodeProcess('C:\\Dev\\workspace\\toolchains\\node\\node.exe',
    'c:\\dev\\workspace\\toolchains\\node\\node.exe','win32'),true);
  assert.equal(isPinnedNodeProcess('C:\\Windows\\node.exe',
    'C:\\Dev\\workspace\\toolchains\\node\\node.exe','win32'),false);
  assert.equal(isPinnedNodeProcess('node.exe','C:\\Dev\\node.exe','win32'),false);
  assert.equal(isPinnedNodeProcess('/repo/node','/repo/node','linux'),true);
  assert.equal(isPinnedNodeProcess('/usr/bin/node','/repo/node','linux'),false);
  assert.equal(isPinnedNodeProcess('node','/repo/node','linux'),false);
});

test('post-task revision check blocks false completion evidence',()=>{
  const baseline={branch:'Local',sha:SHA};
  assert.equal(classifyCompletion(baseline,{...baseline},0), 'COMMAND_SUCCEEDED');
  assert.equal(classifyCompletion(baseline,{...baseline},1), 'FAILED');
  assert.equal(classifyCompletion(baseline,{...baseline},0,true), 'FAILED');
  assert.equal(classifyCompletion(baseline,{branch:'Local',sha:'b'.repeat(40)},0), 'SOURCE_CHANGED');
  assert.equal(classifyCompletion(baseline,{branch:'main',sha:SHA},0), 'SOURCE_CHANGED');
  assert.equal(classifyCompletion(baseline,null,0), 'SOURCE_UNVERIFIED');
});

test('task status is emitted after lock release, and uncertain execution is not a false BLOCKED',()=>{
  const script=readFileSync(new URL('./controller.mjs',import.meta.url),'utf8');
  assert.match(script,/const status=classifyCompletion\(source,after,child\.code,Boolean\(child\.error\),child\.interrupted\)/);
  assert.match(script,/emit\(report\('run',outcome\),parsed\.json\)/);
  assert.match(script,/status:'FAILED_UNVERIFIED'/);
  assert.match(script,/completion or lock release failed/);
  assert.doesNotMatch(script,/return blocked\('run','Another developer task may be active or workspace lock is unsafe; no task started'/);
});


test('developer child stays asynchronous and returns its actual exit status',async()=>{
  let resolved=false;
  const running=runOwnedChild(process.execPath,['-e','setTimeout(()=>process.exit(0),140)'],{
    cwd:root,forwardHostSignals:false
  }).then(result=>{resolved=true;return result;});
  await new Promise(resolve=>setTimeout(resolve,20));
  assert.equal(resolved,false,'controller event loop should be responsive while child runs');
  const result=await running;
  assert.equal(result.code,0);
  assert.equal(result.error,null);
  assert.equal(result.interrupted,false);
});

test('nonzero developer subprocess and spawn error never indicate success',async()=>{
  const failure=await runOwnedChild(process.execPath,['-e','process.exit(7)'],{
    cwd:root,forwardHostSignals:false
  });
  assert.equal(failure.code,7);
  assert.equal(classifyCompletion({branch:'Local',sha:SHA},{branch:'Local',sha:SHA},failure.code), 'FAILED');
  const missing=await runOwnedChild(resolve(root,'missing-developer-executable'),[],{
    cwd:root,forwardHostSignals:false
  });
  assert.notEqual(missing.code,0);
  assert.equal(classifyCompletion({branch:'Local',sha:SHA},{branch:'Local',sha:SHA},missing.code,true),'FAILED');
  assert.ok(missing.error);
  assert.equal(classifyCompletion({branch:'Local',sha:SHA},{branch:'Local',sha:SHA},missing.code,true),'FAILED');
});

test('cancellation never creates false PASS; child completion is still awaited',async()=>{
  const cancellation=new AbortController();
  const running=runOwnedChild(process.execPath,['-e','setTimeout(()=>process.exit(0),200)'],{
    cwd:root,forwardHostSignals:false,cancellationSignal:cancellation.signal
  });
  setTimeout(()=>cancellation.abort(),50);
  const result=await running;
  assert.equal(result.interrupted,true);
  assert.equal(classifyCompletion({branch:'Local',sha:SHA},{branch:'Local',sha:SHA},
    result.code,Boolean(result.error),result.interrupted),'INTERRUPTED');
});

function localLockFixture() {
  const base=mkdtempSync(join(tmpdir(),'lazy-controller-lock-'));
  const workspace=join(base,'workspace'),tmp=join(workspace,'tmp');
  mkdirSync(tmp,{recursive:true});
  return {base,lockFile:join(tmp,'developer-controller.lock')};
}

test('exclusive developer lock spans the awaited child lifetime',async()=>{
  const {base,lockFile}=localLockFixture();
  try{
    const first=withBuildLock('frontend.validate',async()=>{
      await new Promise(resolve=>setTimeout(resolve,120));
      return 'finished';
    },lockFile);
    await new Promise(resolve=>setTimeout(resolve,20));
    assert.equal(existsSync(lockFile),true);
    await assert.rejects(withBuildLock('rust.validate',async()=>{},lockFile),{code:'EEXIST'});
    assert.equal(await first,'finished');
    assert.equal(existsSync(lockFile),false);
    assert.equal(await withBuildLock('rust.validate',async()=>42,lockFile),42);
    assert.equal(existsSync(lockFile),false);
  }finally{rmSync(base,{recursive:true,force:true});}
});

test('controller releases its own lock on failed task but never takes over a stale lock',async()=>{
  const {base,lockFile}=localLockFixture();
  try{
    await assert.rejects(withBuildLock('frontend.validate',async()=>{throw Error('fixture failure')},lockFile),
      /fixture failure/);
    assert.equal(existsSync(lockFile),false);
    writeFileSync(lockFile,'existing-stale-lock');
    await assert.rejects(withBuildLock('rust.validate',async()=>42,lockFile),{code:'EEXIST'});
    assert.equal(readFileSync(lockFile,'utf8'),'existing-stale-lock');
  }finally{rmSync(base,{recursive:true,force:true});}
});

test('replaced lock identity is never unlinked as if it belonged to controller',{skip:process.platform==='win32'},async()=>{
  const {base,lockFile}=localLockFixture();
  try{
    await assert.rejects(withBuildLock('frontend.validate',async()=>{
      rmSync(lockFile);
      writeFileSync(lockFile,'other-actor-lock');
    },lockFile),/Developer lock identity changed/);
    assert.equal(readFileSync(lockFile,'utf8'),'other-actor-lock');
  }finally{rmSync(base,{recursive:true,force:true});}
});

test('interruption does not supersede changed HEAD evidence',()=>{
  const before={branch:'Local',sha:SHA};
  assert.equal(classifyCompletion(before,before,0,false,true),'INTERRUPTED');
  assert.equal(classifyCompletion(before,{branch:'Local',sha:'b'.repeat(40)},0,false,true),'SOURCE_CHANGED');
  assert.equal(classifyCompletion(before,null,0,false,true),'SOURCE_UNVERIFIED');
});

test('console Ctrl+C is observed once; Windows child and Minecraft process tree are never force-killed',()=>{
  const script=readFileSync(new URL('./controller.mjs',import.meta.url),'utf8');
  assert.match(script,/const onInterrupt=\(\)=>requestStop\('SIGINT'\)/);
  assert.match(script,/if\(!forwardChild \|\| process\.platform==='win32'\) return/);
  assert.match(script,/const onTerminate=\(\)=>requestStop\('SIGTERM',true\)/);
  assert.match(script,/const onAbort=\(\)=>requestStop\('SIGTERM',true\)/);
  assert.doesNotMatch(script,/\b(?:taskkill|Stop-Process|killTree|process\.kill)\s*\(/i);
});

test('JSON-mode developer child logs go to stderr and do not corrupt the one stdout result',()=>{
  const modulePath=JSON.stringify(fileURLToPath(new URL('./controller.mjs',import.meta.url)));
  const childScript=JSON.stringify('process.stdout.write("CHILD_DIAGNOSTIC\\n")');
  const parentScript=`import {runOwnedChild} from ${modulePath};
    const result=await runOwnedChild(process.execPath,['-e',${childScript}],{
      cwd:process.cwd(),json:true,forwardHostSignals:false
    });
    console.log(JSON.stringify({status:result.code===0&&!result.error?'COMMAND_SUCCEEDED':'FAILED'}));`;
  const output=spawnSync(process.execPath,['--input-type=module','-e',parentScript],{
    encoding:'utf8',timeout:10000,maxBuffer:8192
  });
  assert.equal(output.status,0,output.stderr);
  assert.equal(JSON.parse(output.stdout.trim()).status,'COMMAND_SUCCEEDED');
  assert.match(output.stderr,/CHILD_DIAGNOSTIC/);
  assert.doesNotMatch(output.stdout,/CHILD_DIAGNOSTIC/);
});
