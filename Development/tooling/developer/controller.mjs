// Single AI-facing developer entrypoint. Existing PowerShell/npm/Cargo/Tauri
// owners remain the only command executors. No arbitrary shell execution.
import {lstatSync, fstatSync, readFileSync, openSync, closeSync, unlinkSync, writeSync, constants} from 'node:fs';
import {spawn, spawnSync} from 'node:child_process';
import {dirname, join, resolve, isAbsolute, posix, win32} from 'node:path';
import {fileURLToPath} from 'node:url';

export const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const REGISTRY = resolve(dirname(fileURLToPath(import.meta.url)), 'tasks.json');
const DEV_SCRIPT = join(REPO, 'Development/tooling/windows-toolchain/dev.ps1');
const REPOSITORY_CHECK = join(REPO, 'Development/tooling/repository/check-structure.mjs');
const ARCHIVE_RETIRE = join(REPO, 'Development/tooling/storage/archive-retire.mjs');
const NODE_EXE = join(REPO, 'workspace/toolchains/node/node.exe');
const PWSH_EXE = join(REPO, 'workspace/toolchains/powershell/pwsh.exe');
const LOCK_FILE = join(REPO, 'workspace/tmp/developer-controller.lock');

// Audited command vocabulary, not a way to execute arbitrary programs.
// Task ids, descriptions and routing live in tasks.json only.
const POLICY = Object.freeze({
  'doctor': {risk:'local-setup', approval:false},
  'setup': {risk:'dependency-write', approval:true},
  'lock': {risk:'source-write', approval:true},
  'repository-structure': {risk:'read', approval:false},
  'check:frontend': {risk:'build', approval:false},
  'check:rust': {risk:'build', approval:false},
  'check': {risk:'build', approval:false},
  'preflight': {risk:'dependency-write', approval:true},
  'test': {risk:'build', approval:false},
  'dev': {risk:'interactive', approval:false},
  'package': {risk:'package-write', approval:true},
  'storage': {risk:'read', approval:false},
  'retire-archive': {risk:'archive-write', approval:true},
  'bench:frontend': {risk:'build', approval:false},
  'bench:rust': {risk:'build', approval:false},
});
const STORAGE_MODES = new Set(['report','explain','plan','cargo']);
const ID = /^[a-z][a-z0-9]*(?:[.-][a-z0-9]+)*$/;
const SHA = /^[a-f0-9]{40}$/;
const SCOPE = new Set(['environment','repository','frontend','rust','source','desktop','package','storage','benchmark']);

export function validatedRegistry(input) {
  if(!input || input.schema!==1 || !Array.isArray(input.tasks) ||
     input.tasks.length<1 || input.tasks.length>50) throw Error('Invalid developer task registry');
  const seen=new Set();
  return Object.freeze(input.tasks.map(value=>{
    if(!value || typeof value.id!=='string' || !ID.test(value.id) || seen.has(value.id) ||
       typeof value.description!=='string' || value.description.length<8 || value.description.length>250 ||
       typeof value.command!=='string' || !Object.hasOwn(POLICY,value.command) ||
       !SCOPE.has(value.scope) || Object.keys(value).some(k=>!['id','description','command','scope','mode'].includes(k))) {
      throw Error('Invalid or duplicate developer task entry');
    }
    if(value.command==='storage' ? !STORAGE_MODES.has(value.mode) : value.mode!==undefined) {
      throw Error('Invalid fixed task mode');
    }
    seen.add(value.id);
    return Object.freeze({...value,risk:POLICY[value.command].risk,
      requiresApproval:POLICY[value.command].approval});
  }));
}

export function classifyPath(path) {
  if(typeof path!=='string' || !path || path.length>512 || path.includes('\\') ||
     isAbsolute(path) || path.startsWith('/') || path.includes('\0')) return 'unknown';
  const clean=posix.normalize(path);
  if(clean!==path || clean==='..' || clean.startsWith('../') || path.startsWith('-')) return 'unknown';
  if(/^(?:Data|App|workspace|\.git)(?:\/|$)/i.test(path)) return 'protected';
  // Markdown guides and skill instructions are not compiled inputs, even when
  // stored beside native source. Do not downgrade executable or unknown files
  // simply because they live beneath docs/, planning/, or Archive/.
  if(/^(?:docs|planning|Archive)\/.*\.md$/.test(path) ||
     /^(?:README|CONTRIBUTING|SECURITY|CONTEXT|GITHUB_RULES|AGENTS)\.md$/.test(path) ||
     /(?:^|\/)(?:README|AGENTS)\.md$/.test(path) ||
     /^\.agents\/skills\/[^/]+\/SKILL\.md$/.test(path)) return 'docs';
  // Build configuration and dependency graph edits can invalidate artifacts
  // across both manifests and profiles; do not classify them as Rust-only.
  if(/(?:^|\/)(?:Cargo\.toml|Cargo\.lock|build\.rs|rust-toolchain(?:\.toml)?)$/.test(path) ||
     path.includes('/.cargo/config') || path.startsWith('.cargo/')) return 'unknown';
  if(path.startsWith('Development/tooling/acceptance/windows/bootstrap-ordering/')) return 'native';
  if(path.startsWith('Development/apps/desktop/src-tauri/src/') ||
     path.startsWith('Development/engine/runtime-core/src/')) return 'rust';
  if(path.startsWith('Development/apps/desktop/src/') ||
     path.startsWith('Development/apps/desktop/test/')) return 'frontend';
  return 'unknown';
}

export function planForPaths(paths,revision) {
  if(!Array.isArray(paths) || !paths.every(x=>typeof x==='string') || paths.length>4096) {
    throw Error('Invalid bounded file selection');
  }
  const kinds=[...new Set(paths.map(classifyPath))].sort();
  if(kinds.includes('protected')) return {
    status:'BLOCKED',sourceSha:revision,changedCount:paths.length,scopes:kinds,
    taskIds:[],reason:'Protected application, account or workspace paths are not eligible for source-routing.'};
  if(kinds.length===0) return {status:'READY',sourceSha:revision,changedCount:0,
    scopes:[],taskIds:[],reason:'No changed source detected; do not rebuild just to create evidence.'};
  // Documentation normally accompanies code changes and must not force a
  // second unnecessary Rust dependency graph or full test suite.
  const executableScopes=kinds.filter(scope=>scope!=='docs');
  if(executableScopes.length===0) return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['repository.validate'],reason:'Documentation-only structural validation.'};
  if(executableScopes.length===1 && executableScopes[0]==='frontend') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['frontend.validate'],reason:'Frontend and documentation validation; no Rust check.'};
  if(executableScopes.length===1 && executableScopes[0]==='rust') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['rust.validate'],reason:'Scoped Rust check; full tests still required for candidate packaging.'};
  return {status:'READY',sourceSha:revision,changedCount:paths.length,scopes:kinds,
    taskIds:['source.validate'],reason:'Mixed, native, configuration or unknown source; use full verification. Native acceptance remains separate.'};
}

export function legacyArgs(task) {
  if(!task || !Object.hasOwn(POLICY,task.command)) throw Error('Unregistered executor');
  if(task.command==='repository-structure') return {type:'node',args:[REPOSITORY_CHECK]};
  if(task.command==='retire-archive') return {type:'node',args:[ARCHIVE_RETIRE]};
  const args=['-NoLogo','-NoProfile','-NonInteractive','-File',DEV_SCRIPT,task.command];
  if(task.command==='storage') args.push(task.mode);
  return {type:'pwsh',args};
}

function git(args) {
  const child=spawnSync('git',args,{cwd:REPO,encoding:'utf8',shell:false,
    timeout:10000,maxBuffer:256*1024,windowsHide:true});
  if(child.error || child.status!==0 || typeof child.stdout!=='string') {
    throw Error('Git metadata unavailable; developer control must fail closed');
  }
  return child.stdout;
}
function sourceInfo() {
  const branch=git(['branch','--show-current']).trim();
  const sha=git(['rev-parse','--verify','HEAD']).trim();
  if(!SHA.test(sha)) throw Error('Invalid source SHA');
  return {branch,sha};
}
function changedPaths(readGit=git) {
  const changed=readGit(['diff','--name-only','-z','HEAD']).split('\0').filter(Boolean);
  const untracked=readGit(['ls-files','--others','--exclude-standard','-z']).split('\0').filter(Boolean);
  if(changed.length+untracked.length>4096) throw Error('Too many changes for bounded planning');
  return [...new Set([...changed,...untracked])];
}
// The ordinary quick route inspects the working tree, or HEAD^ on a clean
// checkout. An explicit ancestor baseline covers an entire development batch
// without a second task runner or a persistent "last validated" state file.
export function automaticChangeSet(baseSha=null,readGit=git) {
  const pending=changedPaths(readGit);
  if(baseSha!==null) {
    if(!SHA.test(baseSha)) throw Error('Invalid exact base SHA');
    const ancestor=readGit(['merge-base',baseSha,'HEAD']).trim();
    if(ancestor!==baseSha) throw Error('Base commit must be an ancestor of current Local HEAD');
    const committed=readGit(['diff','--no-renames','--name-only','-z',baseSha,'HEAD','--']).split('\0').filter(Boolean);
    const paths=[...new Set([...committed,...pending])];
    if(paths.length>4096) throw Error('Too many changes for bounded planning');
    return {paths,basis:'base-commit',baseSha};
  }
  if(pending.length) return {paths:pending,basis:'working-tree'};
  const committed=readGit(['diff','--name-only','-z','HEAD^','HEAD','--']).split('\0').filter(Boolean);
  if(committed.length>4096) throw Error('Too many committed changes for bounded routing');
  return {paths:[...new Set(committed)],basis:'latest-commit'};
}
function ordinary(path) {
  try {const info=lstatSync(path);return info.isFile()&&!info.isSymbolicLink();}
  catch {return false;}
}
// Node's version alone does not prove the current executable is the reviewed
// checkout-local runtime. Compare normalized absolute paths, never PATH lookup.
export function isPinnedNodeProcess(actual,expected,platform=process.platform) {
  if(typeof actual!=='string' || typeof expected!=='string') return false;
  if(platform==='win32') return win32.isAbsolute(actual) && win32.isAbsolute(expected) &&
    win32.normalize(actual).toLowerCase()===win32.normalize(expected).toLowerCase();
  return isAbsolute(actual) && isAbsolute(expected) && resolve(actual)===resolve(expected);
}
// A child process exit is not revision-scoped evidence if Local HEAD changed
// while it ran or the final Git check failed.
export function classifyCompletion(before,after,exitCode,spawnError=false,interrupted=false) {
  if(!after) return 'SOURCE_UNVERIFIED';
  if(before.branch!=='Local' || after.branch!=='Local' || before.sha!==after.sha) return 'SOURCE_CHANGED';
  if(interrupted) return 'INTERRUPTED';
  return spawnError || exitCode!==0 ? 'FAILED' : 'COMMAND_SUCCEEDED';
}
function report(operation,data) {return {schema:1,operation,...data};}
function usage() {
  return 'Usage: DEV.cmd inspect|catalog|plan [--json] [--base-sha <ancestor-40-char-SHA>] [--paths <source paths>...]\n'+
    '       DEV.cmd run auto --expect-sha <40-char Local SHA> [--base-sha <ancestor-40-char-SHA>] [--json]\n'+
    '       DEV.cmd run <task-id> --expect-sha <40-char Local SHA> [--approve <task-id>] [--json]\n'+
    '       DEV.cmd bootstrap (existing reviewed opt-in setup)';
}
export function parseFlags(tokens,operation) {
  const result={json:false,paths:[],taskId:null,expectSha:null,approval:null,baseSha:null};
  const rest=[...tokens];
  if(operation==='run') {
    result.taskId=rest.shift();
    if(!result.taskId || !ID.test(result.taskId)) throw Error('Invalid task id');
  }
  const seen=new Set();
  for(let i=0;i<rest.length;i++) {
    const token=rest[i];
    if(token==='--json' && !seen.has(token)){result.json=true;seen.add(token);continue;}
    if((operation==='plan'||operation==='run') && token==='--base-sha' && !seen.has(token)) {
      const base=rest[++i];
      if(!SHA.test(base??'')) throw Error('--base-sha requires one exact 40-character commit SHA');
      result.baseSha=base;seen.add(token);continue;
    }
    if(operation==='plan' && token==='--paths' && !seen.has(token)) {
      result.paths=rest.slice(i+1);
      if(!result.paths.length || result.paths.some(path=>path.startsWith('--'))) throw Error('--paths requires source paths only');
      break;
    }
    if(operation==='run' && token==='--expect-sha' && !seen.has(token)) {
      result.expectSha=rest[++i];seen.add(token);continue;
    }
    if(operation==='run' && token==='--approve' && !seen.has(token)) {
      result.approval=rest[++i];seen.add(token);continue;
    }
    throw Error('Unexpected or repeated developer controller argument');
  }
  if(result.baseSha!==null && result.paths.length) throw Error('--base-sha cannot be combined with --paths');
  if(result.baseSha!==null && operation==='run' && result.taskId!=='auto') {
    throw Error('--base-sha is only supported by run auto');
  }
  return result;
}

function startup() {
  const pinned=JSON.parse(readFileSync(join(REPO,'Development/toolchain.json'),'utf8'));
  if(process.versions.node!==pinned.node.version) throw Error('Pinned workspace Node version required; no global fallback');
  if(!ordinary(NODE_EXE)) throw Error('Workspace-local Node executable is missing or linked');
  if(!isPinnedNodeProcess(process.execPath,NODE_EXE)) throw Error('Controller must run through pinned DEV.cmd Node; global Node is forbidden');
  return validatedRegistry(JSON.parse(readFileSync(REGISTRY,'utf8')));
}
function toolPresence() {
  return {node:ordinary(NODE_EXE),powershell:ordinary(PWSH_EXE),
    cargo:ordinary(join(REPO,'workspace/toolchains/cargo/bin/cargo.exe'))};
}
function emit(payload,json) {console.log(json?JSON.stringify(payload):JSON.stringify(payload,null,2));}
function blocked(operation,reason,json) {
  emit(report(operation,{status:'BLOCKED',reason,proof:'NOT_EXECUTED'}),json);
  return 3;
}
// Owned developer executor only. Never manage the Minecraft process tree.
// Console Ctrl+C reaches the child independently: do not send it twice.
// On Windows child.kill(SIGINT/TERM) is abrupt; only observe termination
// and preserve a stale lock if the child fails to exit.
export function runOwnedChild(executable,args,{cwd,json=false,cancellationSignal=null,forwardHostSignals=true}={}) {
  return new Promise(resolveResult=>{
    let child=null,childError=null,interrupted=false,closed=false;
    const requestStop=(signalName,forwardChild=false)=>{
      if(closed || interrupted) return;
      interrupted=true;
      if(!forwardChild || process.platform==='win32') return; // No duplicate Ctrl+C or forced Windows termination.
      try {
        if(child?.pid && child.exitCode===null && child.signalCode===null) child.kill(signalName);
      } catch {}
    };
    const onInterrupt=()=>requestStop('SIGINT');
    const onTerminate=()=>requestStop('SIGTERM',true);
    const onAbort=()=>requestStop('SIGTERM',true);
    const cleanup=()=>{
      process.removeListener('SIGINT',onInterrupt);
      process.removeListener('SIGTERM',onTerminate);
      cancellationSignal?.removeEventListener('abort',onAbort);
    };
    if(forwardHostSignals){
      process.on('SIGINT',onInterrupt);
      process.on('SIGTERM',onTerminate);
    }
    cancellationSignal?.addEventListener('abort',onAbort,{once:true});
    try {
      child=spawn(executable,args,{cwd,shell:false,windowsHide:false,
        stdio:json?['inherit',process.stderr,'inherit']:'inherit'});
      child.once('error',error=>{childError=error;});
      child.once('close',(code,signal)=>{
        closed=true;cleanup();
        resolveResult({code,signal,error:childError,interrupted});
      });
      if(cancellationSignal?.aborted) onAbort();
    }catch(error){
      closed=true;cleanup();
      resolveResult({code:null,signal:null,error,interrupted});
    }
  });
}

// One exclusive lock across the entire awaited developer child lifetime.
// The lock belongs to the creator; never take over an unknown stale lock.
export async function withBuildLock(taskId,callback,lockFile=LOCK_FILE) {
  const tmp=dirname(lockFile),workspace=dirname(tmp);
  for(const dir of [workspace,tmp]){
    const stat=lstatSync(dir);
    if(!stat.isDirectory() || stat.isSymbolicLink()) throw Error('Developer workspace lock directory must be ordinary');
  }
  const fd=openSync(lockFile,constants.O_CREAT|constants.O_EXCL|constants.O_WRONLY,0o600);
  let original;
  try {
    original=fstatSync(fd);
    writeSync(fd,JSON.stringify({taskId,pid:process.pid})+'\n');
    return await callback();
  }finally{
    closeSync(fd);
    // If identity admission failed, preserve the lock for explicit inspection.
    if(!original) throw Error('Developer lock identity unavailable; manual review required');
    const current=lstatSync(lockFile);
    if(!current.isFile() || current.isSymbolicLink() || current.nlink!==1 ||
       current.dev!==original.dev || current.ino!==original.ino) {
      throw Error('Developer lock identity changed: preserve it for manual review');
    }
    unlinkSync(lockFile);
  }
}
async function runTask(task,parsed,source,selection=null) {
  if(source.branch!=='Local') return blocked('run','Task execution requires branch Local',parsed.json);
  if(!parsed.expectSha || !SHA.test(parsed.expectSha) || source.sha!==parsed.expectSha) {
    return blocked('run','Exact current Local SHA is required; re-inspect the checkout',parsed.json);
  }
  if(task.requiresApproval && parsed.approval!==task.id) return blocked('run',
    'Operator approval required for this task; supply --approve with the exact task id only after explicit approval',parsed.json);
  if(!task.requiresApproval && parsed.approval!==null) return blocked('run',
    'Approval flag is reserved for explicit high-impact tasks',parsed.json);
  const execution=legacyArgs(task);
  const executable=execution.type==='node'?NODE_EXE:PWSH_EXE;
  if(!ordinary(executable)) return blocked('run','Pinned workspace executor missing or linked',parsed.json);
  let attempted=false;
  const invoke=async()=>{
    const latest=sourceInfo();
    if(latest.branch!=='Local' || latest.sha!==source.sha) {
      return {status:'BLOCKED',taskId:task.id,sourceSha:source.sha,
        reason:'Local HEAD changed before execution; task not started',proof:'NOT_EXECUTED'};
    }
    attempted=true;
    const child=await runOwnedChild(executable,execution.args,{cwd:join(REPO,'Development'),json:parsed.json});
    let after;
    try {after=sourceInfo();} catch {after=null;}
    const status=classifyCompletion(source,after,child.code,Boolean(child.error),child.interrupted);
    return {status,taskId:task.id,sourceSha:source.sha,exitCode:child.code??null,
      proof:status==='COMMAND_SUCCEEDED'?'COMMAND_EXIT_AND_HEAD_ONLY':'NOT_VERIFIED',
      note:'Only the committed HEAD was rechecked. Uncommitted source and Windows/game acceptance are not certified.',
      ...(selection ? {selectionBasis:selection.basis,selectedChangeCount:selection.paths.length,
        ...(selection.baseSha ? {baseSha:selection.baseSha} : {})} : {})};
  };
  let outcome;
  try {outcome=await (task.risk==='read'?invoke():withBuildLock(task.id,invoke));}
  catch {
    if(!attempted) return blocked('run','Task could not start: developer lock or Git metadata is unsafe',parsed.json);
    emit(report('run',{status:'FAILED_UNVERIFIED',taskId:task.id,sourceSha:source.sha,
      proof:'OUTCOME_UNVERIFIED',reason:'Task may have run but completion or lock release failed. Inspect manually; no automatic lock recovery.'}),parsed.json);
    return 1;
  }
  emit(report('run',outcome),parsed.json);
  return outcome.status==='COMMAND_SUCCEEDED'?0:outcome.status==='BLOCKED'?3:1;
}
export async function main(argv=process.argv.slice(2)) {
  const [operation,...arguments_]=argv;
  if(!['inspect','catalog','plan','run'].includes(operation)) {
    console.error(usage());return 2;
  }
  let parsed;
  try {parsed=parseFlags(arguments_,operation);}catch(error){console.error(error.message+'\n'+usage());return 2;}
  let tasks;
  try {tasks=startup();}
  catch(error){return blocked(operation,'Pinned Node or developer task registry is unavailable/invalid',parsed.json);}
  if(operation==='catalog') {
    emit(report(operation,{status:'READY',tasks:tasks.map(({id,description,scope,risk,requiresApproval})=>
      ({id,description,scope,risk,requiresApproval})),proof:'REGISTRY_ONLY'}),parsed.json);
    return 0;
  }
  let source;
  try {source=sourceInfo();}
  catch(error){return blocked(operation,error.message,parsed.json);}
  if(operation==='inspect') {
    let paths;
    try {paths=changedPaths();}catch(error){return blocked(operation,error.message,parsed.json);}
    emit(report(operation,{status:source.branch==='Local'?'READY':'BLOCKED',
      source:{branch:source.branch,sha:source.sha},changedCount:paths.length,
      scopes:[...new Set(paths.map(classifyPath))].sort(),toolchainPresence:toolPresence(),
      proof:'GIT_AND_FILE_METADATA_ONLY',note:'Presence is not version or execution proof.'}),parsed.json);
    return source.branch==='Local'?0:3;
  }
  if(operation==='plan') {
    if(source.branch!=='Local') return blocked(operation,'Planning requires branch Local',parsed.json);
    let selection;
    try {selection=parsed.paths.length
      ?{paths:parsed.paths,basis:'declared-paths'}:automaticChangeSet(parsed.baseSha);}
    catch(error){return blocked(operation,error.message,parsed.json);}
    const planned=planForPaths(selection.paths,source.sha);
    emit(report(operation,{...planned,basis:selection.basis,
      ...(selection.baseSha ? {baseSha:selection.baseSha} : {}),
      sourceBranch:source.branch,proof:'STATIC_RECOMMENDATION_ONLY'}),parsed.json);
    return source.branch==='Local'&&planned.status!=='BLOCKED'?0:3;
  }
  let requested=parsed.taskId;
  let autoSelection=null;
  if(requested==='auto') {
    if(source.branch!=='Local')return blocked(operation,'Automatic validation requires Local',parsed.json);
    let selection;
    try {selection=automaticChangeSet(parsed.baseSha);}
    catch(error){return blocked(operation,error.message,parsed.json);}
    autoSelection=selection;
    const recommendation=planForPaths(selection.paths,source.sha);
    if(recommendation.status!=='READY') return blocked(operation,
      'Automatic validation blocked by protected or untrusted paths',parsed.json);
    if(!parsed.expectSha || parsed.expectSha!==source.sha || !SHA.test(parsed.expectSha)) {
      return blocked(operation,'Automatic validation requires the exact current Local SHA',parsed.json);
    }
    if(recommendation.taskIds.length===0) {
      emit(report(operation,{status:'SKIPPED_NO_CHANGES',sourceSha:source.sha,
        selectionBasis:selection.basis,...(selection.baseSha ? {baseSha:selection.baseSha} : {}),
        proof:'STATIC_RECOMMENDATION_ONLY',reason:'No changed source; no build required.'}),parsed.json);
      return 0;
    }
    if(recommendation.taskIds.length!==1) return blocked(operation,'Invalid automatic task plan',parsed.json);
    requested=recommendation.taskIds[0];
  }
  const task=tasks.find(t=>t.id===requested);
  if(!task) return blocked(operation,'Unknown task id; use catalog for available capabilities',parsed.json);
  if(parsed.taskId==='auto' && task.requiresApproval)return blocked(operation,'Automatic mode cannot run consent-gated tasks',parsed.json);
  return await runTask(task,parsed,source,autoSelection);
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {process.exitCode=await main();}
  catch(error){console.error('Developer controller failed closed: '+(error?.code??error?.message??'unknown error'));process.exitCode=3;}
}
