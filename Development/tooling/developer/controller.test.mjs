import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {dirname,resolve,join} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {validatedRegistry,classifyPath,planForPaths,planForProfile,qualityReviewForPaths,automaticChangeSet,legacyArgs,parseFlags,parseSmokeFlags,isPinnedNodeProcess,classifyCompletion,runOwnedChild,withBuildLock} from './controller.mjs';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
const registry=JSON.parse(readFileSync(new URL('./tasks.json',import.meta.url),'utf8'));
const tasks=validatedRegistry(registry);
const SHA='a'.repeat(40);

test('catalog defines unique bounded fixed tasks and never exposes shell executors',()=>{
  assert.ok(tasks.length>=12);
  assert.equal(new Set(tasks.map(t=>t.id)).size,tasks.length);
  assert.deepEqual(tasks.filter(t=>t.requiresApproval).map(t=>t.id).sort(),
    ['build.candidate','cargo.lock','environment.setup','source.preflight','storage.retire-archive'].sort());
  assert.equal(tasks.find(t=>t.id==='storage.plan').risk,'read');
  for(const task of tasks) assert.ok(['node','pwsh'].includes(legacyArgs(task).type));
  assert.ok(!tasks.some(t=>t.command==='g0'));
});

test('retired Cargo cache cleanup is opt-in and scoped to reviewed executable',()=>{
  const task=tasks.find(t=>t.id==='storage.retire-archive');
  assert.ok(task);
  assert.equal(task.risk,'archive-write');
  assert.equal(task.requiresApproval,true);
  const cmd=legacyArgs(task);
  assert.equal(cmd.type,'node');
  assert.equal(cmd.args.length,1);
  assert.ok(cmd.args[0].endsWith('tooling/storage/archive-retire.mjs'));
  assert.equal(parseFlags(['storage.retire-archive','--expect-sha',SHA,'--approve','storage.retire-archive'],'run').approval,'storage.retire-archive');
  assert.deepEqual(planForPaths(['Development/tooling/storage/archive-retire.mjs'],SHA).taskIds,['source.validate']);
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

test('operator-only smoke commands require revision pinning, safe IDs and explicit PASS report',()=>{
  const id='20261103T123000Z-aaaaaaaa-smoke-1234abcd';
  assert.equal(parseSmokeFlags(['start','--expect-sha',SHA]).action,'start');
  assert.equal(parseSmokeFlags(['record','--run',id,'--case','LT-01','--result','PASS',
    '--observed','--expect-sha',SHA]).reportedStatus,'PASS');
  assert.equal(parseSmokeFlags(['status','--run',id,'--json']).json,true);
  assert.throws(()=>parseSmokeFlags(['record','--run',id,'--case','LT-01','--result','PASS',
    '--expect-sha',SHA]),/operator-confirmed PASS/);
  assert.throws(()=>parseSmokeFlags(['start','--expect-sha','main']),/exact --expect-sha/);
  assert.throws(()=>parseSmokeFlags(['status','--run','../../Data']),/bounded run ID/);
  assert.throws(()=>parseSmokeFlags(['record','--run',id,'--case','LT-14','--result','PASS',
    '--observed','--expect-sha',SHA]),/LT-01..LT-13/);
  assert.throws(()=>parseSmokeFlags(['status','--run',id,'--observed']),/Unexpected/);
  assert.throws(()=>parseSmokeFlags(['start','--expect-sha',SHA,'--expect-sha',SHA]),/repeated/);
});

test('first-run profile uses a real baseline regardless of a docs-only latest commit',()=>{

  const first=planForProfile('local-smoke',SHA);
  assert.equal(first.status,'READY');
  assert.equal(first.profile,'local-smoke');
  assert.deepEqual(first.taskIds,['source.quick']);
  assert.deepEqual(first.manualCaseIds,Array.from({length:13},(_,i)=>'LT-'+String(i+1).padStart(2,'0')));
  assert.equal(first.manualExecution,'OPERATOR_ONLY');
  assert.ok(!first.taskIds.includes('dev.start'));
  assert.equal(tasks.find(t=>t.id==='source.quick').requiresApproval,false);
  assert.equal(tasks.find(t=>t.id==='source.quick').command,'check:quick');
  assert.throws(()=>planForProfile('local-smoke',SHA,['docs/README.md']),/never route/);
});

test('local-smoke is a self-contained ordered risk-based evidence plan, not gameplay execution',()=>{
  const plan=planForProfile('local-smoke',SHA);
  assert.equal(plan.checkpoints.length,13);
  assert.deepEqual(plan.checkpoints.map(step=>step.id),plan.manualCaseIds);
  assert.equal(new Set(plan.checkpoints.map(step=>step.id)).size,13);
  assert.equal(plan.checkpoints[1].evidenceKind,'SOURCE_QUICK');
  assert.equal(plan.checkpoints[1].stopOnFail,true);
  assert.equal(plan.checkpoints.at(-1).stopOnFail,false);
  for(const step of plan.checkpoints){
    assert.ok(step.action.length>16 && step.passCondition.length>20);
    assert.match(step.legacyCases,/^STD-/);
    assert.equal(typeof step.stopOnFail,'boolean');
    assert.ok(!step.action.includes('Full Clear'));
  }
  assert.equal(plan.manualExecution,'OPERATOR_ONLY');
  assert.ok(!plan.taskIds.includes('dev.start'));
});
 
test('progressive profiles never auto-execute gameplay, AI or packaging',()=>{
  assert.deepEqual(planForProfile('fast',SHA,['docs/README.md']).taskIds,['repository.validate']);
  assert.deepEqual(planForProfile('five-client',SHA).taskIds,[]);
  assert.equal(planForProfile('five-client',SHA).manualCaseIds.length,4);
  assert.deepEqual(planForProfile('release',SHA).taskIds,['source.preflight','build.candidate']);
  assert.ok(planForProfile('release',SHA).prerequisites.some(x=>/approval/.test(x)));
  assert.deepEqual(planForProfile('experimental',SHA).taskIds,[]);
  assert.throws(()=>planForProfile('unknown',SHA),/Unknown/);
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

test('docs accompanying frontend or Rust source do not force redundant full checks',()=>{
  assert.deepEqual(planForPaths(['planning/development.md','Development/apps/desktop/src/App.svelte'],SHA).taskIds,['frontend.validate']);
  assert.deepEqual(planForPaths(['docs/system/architecture.md','Development/engine/runtime-core/src/lib.rs'],SHA).taskIds,['rust.validate']);
  assert.deepEqual(planForPaths(['planning/development.md','docs/system/architecture.md'],SHA).taskIds,['repository.validate']);
  assert.deepEqual(planForPaths(['planning/development.md','Development/tooling/developer/tasks.json'],SHA).taskIds,['source.validate']);
  assert.deepEqual(planForPaths(['planning/development.md','Development/apps/desktop/src/App.svelte','Development/engine/runtime-core/src/lib.rs'],SHA).taskIds,['source.validate']);
});

test('documentation and agent skill edits avoid native builds without hiding executable changes',()=>{
  const guides=[
    'docs/system/architecture.md',
    'planning/development.md',
    'Archive/experiments/minecraft-education-platform-research.md',
    '.agents/skills/emulator-development/SKILL.md',
    'Development/apps/desktop/AGENTS.md',
    'Development/tooling/acceptance/windows/bootstrap-ordering/README.md',
  ];
  for(const file of guides) assert.equal(classifyPath(file),'docs',file);
  assert.deepEqual(planForPaths(guides,SHA).taskIds,['repository.validate']);
  assert.deepEqual(planForPaths([...guides,'Development/apps/desktop/src/LazyTesterApp.svelte'],SHA).taskIds,['frontend.validate']);
  assert.deepEqual(planForPaths([...guides,'Development/engine/runtime-core/src/lib.rs'],SHA).taskIds,['rust.validate']);

  for(const file of [
    'docs/validation/helper.mjs',
    'planning/generator.js',
    'Archive/experiments/helper.ps1',
    '.agents/skills/emulator-development/tools.mjs',
    'Development/tooling/acceptance/windows/bootstrap-ordering/compat.cpp',
    'Development/toolchain.json',
  ]){
    assert.notEqual(classifyPath(file),'docs',file);
    assert.deepEqual(planForPaths([file],SHA).taskIds,['source.validate'],file);
  }
  assert.equal(classifyPath('Data/README.md'),'protected');
  assert.equal(planForPaths(['Data/README.md'],SHA).status,'BLOCKED');
});

test('Cargo manifests, locks and native build scripts always require full verification',()=>{
  for(const path of [
    'Development/apps/desktop/src-tauri/Cargo.toml',
    'Development/apps/desktop/src-tauri/Cargo.lock',
    'Development/apps/desktop/src-tauri/build.rs',
    'Development/engine/runtime-core/Cargo.toml',
    'Development/engine/runtime-core/Cargo.lock',
    'Development/engine/runtime-core/build.rs',
    'Development/toolchain.json',
    'Development/apps/desktop/src-tauri/tauri.lazy-tester.conf.json',
  ]){
    assert.deepEqual(planForPaths([path,'planning/development.md'],SHA).taskIds,['source.validate'],path);
  }
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
  assert.equal(parseFlags(['--profile','local-smoke','--json'],'plan').profile,'local-smoke');
  assert.equal(parseFlags(['--profile','fast','--paths','docs/README.md'],'plan').profile,'fast');
  assert.throws(()=>parseFlags(['--profile','unknown'],'plan'),/--profile requires/);
  assert.throws(()=>parseFlags(['--profile','local-smoke','--base-sha',SHA],'plan'),/Manual profiles/);
  assert.throws(()=>parseFlags(['--profile','release','--paths','docs/README.md'],'plan'),/Manual profiles/);
  assert.throws(()=>parseFlags(['auto','--profile','local-smoke','--expect-sha',SHA],'run'),/Unexpected/);
  assert.equal(parseFlags(['--json','--paths','docs/README.md'],'plan').paths[0],'docs/README.md');
  assert.equal(parseFlags(['frontend.validate','--expect-sha',SHA],'run').expectSha,SHA);
  assert.equal(parseFlags(['--base-sha',SHA,'--json'],'plan').baseSha,SHA);
  assert.equal(parseFlags(['auto','--expect-sha',SHA,'--base-sha',SHA],'run').baseSha,SHA);
  assert.throws(()=>parseFlags(['auto','--base-sha','bad'],'run'),/40-character/);
  assert.throws(()=>parseFlags(['auto','--base-sha',SHA,'--base-sha',SHA],'run'),/Unexpected/);
  assert.throws(()=>parseFlags(['frontend.validate','--base-sha',SHA],'run'),/run auto/);
  assert.throws(()=>parseFlags(['--base-sha',SHA,'--paths','docs/README.md'],'plan'),/cannot be combined/);
  assert.throws(()=>parseFlags(['--paths','docs/README.md','--base-sha',SHA],'plan'),/source paths only/);
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

test('quality review is deterministic, bounded and explicitly does not execute external harnesses',()=>{
  const paths=[
    'Development/tooling/agent/check-education-native-hud-source.mjs',
    'Development/apps/desktop/src-tauri/src/engine/process_identity.rs'
  ];
  const review=qualityReviewForPaths(paths,SHA);
  assert.equal(review.status,'REVIEW_REQUIRED');
  assert.equal(review.sourceSha,SHA);
  assert.deepEqual(review.areas.map(area=>area.id),['education-hud','native-session']);
  assert.deepEqual(review.stages.map(stage=>stage.id),['PROPOSE','PROBE','COMPOSE']);
  assert.ok(review.stages.every(stage=>stage.status==='NOT_EXECUTED'));
  assert.ok(review.areas.every(area=>area.status==='NOT_REVIEWED'));
  assert.deepEqual(review.recommendedValidationTaskIds,['source.validate']);
  assert.match(review.externalIntegrations.rea,/NOT_RUN/);
  assert.equal(review.evidence.minecraftRuntime,'NOT_EXECUTED');
  assert.equal(review.proof,'STATIC_REVIEW_PLAN_ONLY');
  assert.deepEqual(qualityReviewForPaths(paths,SHA),review);
});
test('quality review preserves scoped documentation/no-change semantics',()=>{
  const docs=qualityReviewForPaths(['docs/system/development-discipline.md'],SHA);
  assert.deepEqual(docs.areas.map(area=>area.id),['documentation']);
  assert.deepEqual(docs.recommendedValidationTaskIds,['repository.validate']);
  assert.equal(qualityReviewForPaths([],SHA).status,'NO_CHANGES');
  assert.deepEqual(qualityReviewForPaths([],SHA).stages,[]);
  assert.throws(()=>qualityReviewForPaths([], 'not-a-sha'),/exact source SHA/);
});
test('HUD review routes the checker, fixture, Resource Pack and HUD guide to one owner',()=>{
  const artifacts=[
    'Development/tooling/agent/check-education-native-hud-source.mjs',
    'Development/tooling/agent/education-native-hud-prototype.source.test.mjs',
    'Development/tooling/agent/education-native-hud-prototype/manifest.json',
    'Development/tooling/agent/education-native-hud-prototype/ui/hud_screen.json',
    'docs/system/education-native-coordinate-hud.md'
  ];
  for(const path of artifacts){
    const result=qualityReviewForPaths([path],SHA);
    assert.deepEqual(result.areas.map(area=>area.id),['education-hud'],path);
    assert.equal(result.areas[0].status,'NOT_REVIEWED');
    assert.equal(result.proof,'STATIC_REVIEW_PLAN_ONLY');
    assert.equal(result.evidence.minecraftRuntime,'NOT_EXECUTED');
    assert.deepEqual(result.recommendedValidationTaskIds,
      path.startsWith('docs/')?['repository.validate']:['source.validate']);
  }
  const combined=qualityReviewForPaths(artifacts,SHA);
  assert.equal(combined.areas.length,1);
  assert.equal(combined.changedCount,5);
  assert.deepEqual(combined.stages.map(s=>s.status),
    ['NOT_EXECUTED','NOT_EXECUTED','NOT_EXECUTED']);
  assert.ok(combined.areas[0].reviewChecks.some(check=>check.includes('negative fixtures')));
  assert.match(combined.areas[0].canonicalOwner,/education-native-hud-prototype.source.test.mjs|HUD fixture/);
  assert.deepEqual(combined.recommendedValidationTaskIds,['source.validate']);
  assert.equal(qualityReviewForPaths(['Development/tooling/agent/mcp.mjs'],SHA).areas[0].id,'agent-mcp');
  assert.equal(qualityReviewForPaths(['docs/system/architecture.md'],SHA).areas[0].id,'documentation');
});

test('quality review rejects repeated paths rather than overcounting selected change evidence',()=>{
  const doc='docs/system/education-native-coordinate-hud.md';
  const review=qualityReviewForPaths([doc,doc],SHA);
  assert.equal(review.status,'BLOCKED');
  assert.deepEqual(review.areas,[]);
  assert.deepEqual(review.recommendedValidationTaskIds,[]);
  assert.match(review.reason,/repeated/);
});

test('quality review fails closed for protected, absolute or path traversal inputs',()=>{
  for(const path of ['Data/world','workspace/private','App/launcher.exe',
    '../Data/private','a/../../Data','/etc/passwd','C:/Users/test/world',
    'docs\\escape.md','-unsafe.md']) {
    const review=qualityReviewForPaths([path],SHA);
    assert.equal(review.status,'BLOCKED',path);
    assert.deepEqual(review.areas,[],path);
    assert.deepEqual(review.recommendedValidationTaskIds,[],path);
  }
});
test('review CLI uses the existing read-only controller with strict argument admission',()=>{
  assert.equal(parseFlags(['--json'],'review').json,true);
  assert.deepEqual(parseFlags(['--paths','docs/README.md'],'review').paths,['docs/README.md']);
  assert.equal(parseFlags(['--base-sha',SHA],'review').baseSha,SHA);
  assert.throws(()=>parseFlags(['--profile','fast'],'review'),/Unexpected/);
  assert.throws(()=>parseFlags(['--base-sha',SHA,'--paths','docs/a.md'],'review'),/cannot be combined/);
  assert.throws(()=>parseFlags(['--paths','docs/a.md','--base-sha',SHA],'review'),/source paths only/);
  assert.throws(()=>parseFlags(['--paths'],'review'),/source paths only/);
  assert.throws(()=>parseFlags(['--json','--json'],'review'),/repeated/);
  const launcher=readFileSync(resolve(root,'DEV.cmd'),'utf8');
  assert.match(launcher,/if \/I "%~1"=="review" goto controller/);
});

test('default auto planning stays lightweight on changed files or the most recent commit',()=>{
  const calls=[];
  const readGit=(args)=>{
    calls.push(args);
    const key=JSON.stringify(args);
    if(key===JSON.stringify(['diff','--name-only','-z','HEAD'])) return '';
    if(key===JSON.stringify(['ls-files','--others','--exclude-standard','-z'])) return '';
    if(key===JSON.stringify(['diff','--name-only','-z','HEAD^','HEAD','--'])) return 'docs/README.md\0';
    throw Error('Unexpected Git command: '+key);
  };
  assert.deepEqual(automaticChangeSet(null,readGit),{paths:['docs/README.md'],basis:'latest-commit'});
  assert.equal(calls.length,3);
});

test('explicit ancestor baseline covers all committed changes plus pending edits',()=>{
  const base='b'.repeat(40);
  const replies=new Map([
    [JSON.stringify(['diff','--name-only','-z','HEAD']),'Development/apps/desktop/src/LazyTesterApp.svelte\0'],
    [JSON.stringify(['ls-files','--others','--exclude-standard','-z']),''],
    [JSON.stringify(['merge-base',base,'HEAD']),base+'\n'],
    [JSON.stringify(['diff','--no-renames','--name-only','-z',base,'HEAD','--']),
      'Development/engine/runtime-core/src/lib.rs\0Development/apps/desktop/src/LazyTesterApp.svelte\0'],
  ]);
  const readGit=args=>{
    const key=JSON.stringify(args);
    if(!replies.has(key))throw Error('Unexpected Git command '+key);
    return replies.get(key);
  };
  const selection=automaticChangeSet(base,readGit);
  assert.equal(selection.basis,'base-commit');
  assert.equal(selection.baseSha,base);
  assert.deepEqual(selection.paths,[
    'Development/engine/runtime-core/src/lib.rs',
    'Development/apps/desktop/src/LazyTesterApp.svelte'
  ]);
  assert.deepEqual(planForPaths(selection.paths,SHA).taskIds,['source.validate']);
  replies.set(JSON.stringify(['merge-base',base,'HEAD']),'c'.repeat(40)+'\n');
  assert.throws(()=>automaticChangeSet(base,readGit),/ancestor/);
  assert.throws(()=>automaticChangeSet('untrusted',readGit),/Invalid exact base/);
});

test('empty baseline delta never claims a validation command ran',()=>{
  const base='b'.repeat(40);
  const replies=new Map([
    [JSON.stringify(['diff','--name-only','-z','HEAD']),''],
    [JSON.stringify(['ls-files','--others','--exclude-standard','-z']),''],
    [JSON.stringify(['merge-base',base,'HEAD']),base+'\n'],
    [JSON.stringify(['diff','--no-renames','--name-only','-z',base,'HEAD','--']),''],
  ]);
  const readGit=args=>replies.get(JSON.stringify(args))??(()=>{throw Error('Unexpected git request');})();
  const selection=automaticChangeSet(base,readGit);
  assert.deepEqual(selection.paths,[]);
  assert.deepEqual(planForPaths(selection.paths,SHA).taskIds,[]);
});

test('automatic run remains one allowlisted validation without mutating approvals',()=>{
  const source=readFileSync(new URL('./controller.mjs',import.meta.url),'utf8');
  assert.match(source,/requested==='auto'/);
  assert.match(source,/if\(recommendation\.taskIds\.length!==1\)/);
  assert.match(source,/if\(parsed\.taskId==='auto' && task\.requiresApproval\)/);
  assert.match(source,/SKIPPED_NO_CHANGES/);
  assert.doesNotMatch(source,/requested==='auto'[\s\S]*?runTask\([^,]+,\s*parsed,\s*source\)[\s\S]*?--approve.*automatically generated/i);
});

test('automatic plan never executes unknown or protected paths without safe admission',()=>{
  assert.equal(planForPaths(['Data/profiles.json'],SHA).status,'BLOCKED');
  assert.deepEqual(planForPaths(['Development/apps/desktop/src/LazyTesterApp.svelte'],SHA).taskIds,['frontend.validate']);
  assert.deepEqual(planForPaths(['Development/engine/runtime-core/Cargo.toml'],SHA).taskIds,['source.validate']);
  assert.deepEqual(planForPaths(['Development/tooling/developer/tasks.json'],SHA).taskIds,['source.validate']);
});
