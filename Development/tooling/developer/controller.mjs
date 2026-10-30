// Single AI-facing developer entrypoint. Existing PowerShell/npm/Cargo/Tauri
// owners remain the only command executors. No arbitrary shell execution.
import {lstatSync, readFileSync, openSync, closeSync, unlinkSync, writeSync, constants} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {dirname, join, resolve, isAbsolute, posix} from 'node:path';
import {fileURLToPath} from 'node:url';

export const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const REGISTRY = resolve(dirname(fileURLToPath(import.meta.url)), 'tasks.json');
const DEV_SCRIPT = join(REPO, 'Development/tooling/windows-toolchain/dev.ps1');
const REPOSITORY_CHECK = join(REPO, 'Development/tooling/repository/check-structure.mjs');
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
  if(path.startsWith('Development/tooling/acceptance/windows/bootstrap-ordering/')) return 'native';
  if(path.startsWith('Development/apps/desktop/src-tauri/') ||
     path.startsWith('Development/engine/runtime-core/')) return 'rust';
  if(path.startsWith('Development/apps/desktop/src/') ||
     path.startsWith('Development/apps/desktop/test/')) return 'frontend';
  if(path.startsWith('docs/') || path.startsWith('planning/') ||
     /^(?:README|CONTRIBUTING|SECURITY|CONTEXT|GITHUB_RULES|AGENTS)\.md$/.test(path) ||
     path.endsWith('/README.md')) return 'docs';
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
  if(kinds.length===1 && kinds[0]==='docs') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['repository.validate'],reason:'Documentation-only structural validation.'};
  if(kinds.length===1 && kinds[0]==='frontend') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['frontend.validate'],reason:'Frontend-only validation; no Rust check.'};
  if(kinds.length===1 && kinds[0]==='rust') return {status:'READY',sourceSha:revision,
    changedCount:paths.length,scopes:kinds,taskIds:['rust.validate'],reason:'Scoped Rust check; full tests still required for candidate packaging.'};
  return {status:'READY',sourceSha:revision,changedCount:paths.length,scopes:kinds,
    taskIds:['source.validate'],reason:'Mixed, native, configuration or unknown source; use full verification. Native acceptance remains separate.'};
}

export function legacyArgs(task) {
  if(!task || !Object.hasOwn(POLICY,task.command)) throw Error('Unregistered executor');
  if(task.command==='repository-structure') return {type:'node',args:[REPOSITORY_CHECK]};
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
function changedPaths() {
  const changed=git(['diff','--name-only','-z','HEAD']).split('\0').filter(Boolean);
  const untracked=git(['ls-files','--others','--exclude-standard','-z']).split('\0').filter(Boolean);
  if(changed.length+untracked.length>4096) throw Error('Too many changes for bounded planning');
  return [...new Set([...changed,...untracked])];
}
function ordinary(path) {
  try {const info=lstatSync(path);return info.isFile()&&!info.isSymbolicLink();}
  catch {return false;}
}
function report(operation,data) {return {schema:1,operation,...data};}
function usage() {
  return 'Usage: DEV.cmd inspect|catalog|plan [--json] [--paths <source paths>...]\n'+
    '       DEV.cmd run <task-id> --expect-sha <40-char Local SHA> [--approve <task-id>] [--json]\n'+
    '       DEV.cmd bootstrap (existing reviewed opt-in setup)';
}
export function parseFlags(tokens,operation) {
  const result={json:false,paths:[],taskId:null,expectSha:null,approval:null};
  const rest=[...tokens];
  if(operation==='run') {
    result.taskId=rest.shift();
    if(!result.taskId || !ID.test(result.taskId)) throw Error('Invalid task id');
  }
  const seen=new Set();
  for(let i=0;i<rest.length;i++) {
    const token=rest[i];
    if(token==='--json' && !seen.has(token)){result.json=true;seen.add(token);continue;}
    if(operation==='plan' && token==='--paths' && !seen.has(token)) {
      result.paths=rest.slice(i+1);
      if(!result.paths.length) throw Error('--paths requires source paths');
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
  return result;
}

function startup() {
  const pinned=JSON.parse(readFileSync(join(REPO,'Development/toolchain.json'),'utf8'));
  if(process.versions.node!==pinned.node.version) throw Error('Pinned workspace Node version required; no global fallback');
  if(!ordinary(NODE_EXE)) throw Error('Workspace-local Node executable is missing or linked');
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
function withBuildLock(taskId,callback) {
  const workspace=join(REPO,'workspace');
  const tmp=join(workspace,'tmp');
  for(const dir of [workspace,tmp]) {
    const stat=lstatSync(dir);
    if(!stat.isDirectory()||stat.isSymbolicLink())throw Error('Developer workspace lock directory must be ordinary');
  }
  const fd=openSync(LOCK_FILE,constants.O_CREAT|constants.O_EXCL|constants.O_WRONLY,0o600);
  try {
    writeSync(fd,JSON.stringify({taskId,pid:process.pid})+'\n');
    return callback();
  }finally{
    closeSync(fd);
    unlinkSync(LOCK_FILE); // Only this invocation's freshly created lock.
  }
}
function runTask(task,parsed,source) {
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
  const invoke=()=>{
    const latest=sourceInfo();
    if(latest.branch!=='Local' || latest.sha!==source.sha) {
      return blocked('run','Local HEAD changed before execution; task not started',parsed.json);
    }
    const child=spawnSync(executable,execution.args,{cwd:join(REPO,'Development'),shell:false,
      stdio:parsed.json?['inherit',2,2]:'inherit',windowsHide:false});
    const status=child.error?'FAILED':child.status===0?'COMMAND_SUCCEEDED':'FAILED';
    emit(report('run',{status,taskId:task.id,sourceSha:source.sha,
      exitCode:child.status??null,proof:'COMMAND_EXIT_ONLY',
      note:'Compiler/package exit status is not Minecraft, signing or installed-game acceptance.'}),parsed.json);
    return child.status===0&&!child.error?0:1;
  };
  if(task.risk==='read') return invoke();
  try{return withBuildLock(task.id,invoke);}
  catch(error){return blocked('run','Another developer task may be active or workspace lock is unsafe; no task started',parsed.json);}
}

export function main(argv=process.argv.slice(2)) {
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
    let paths;
    try {paths=parsed.paths.length?parsed.paths:changedPaths();}
    catch(error){return blocked(operation,error.message,parsed.json);}
    const planned=planForPaths(paths,source.sha);
    emit(report(operation,{...planned,basis:parsed.paths.length?'declared-paths':'working-tree',
      sourceBranch:source.branch,proof:'STATIC_RECOMMENDATION_ONLY'}),parsed.json);
    return source.branch==='Local'&&planned.status!=='BLOCKED'?0:3;
  }
  const task=tasks.find(t=>t.id===parsed.taskId);
  if(!task) return blocked(operation,'Unknown task id; use catalog for available capabilities',parsed.json);
  return runTask(task,parsed,source);
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {process.exitCode=main();}
  catch(error){console.error('Developer controller failed closed: '+(error?.code??error?.message??'unknown error'));process.exitCode=3;}
}
