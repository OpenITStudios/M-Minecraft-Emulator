// Team Skills updates: read-only ZIP comparison and source-compatibility gate.
// Runtime modules remain owned by the team repositories. This never syncs,
// installs, modifies Minecraft, applies an update or opens a network socket.
// Operator ZIPs are treated as untrusted until source/lock checks succeed.
import {readFile,lstat,mkdir,open} from 'node:fs/promises';
import {fileURLToPath} from 'node:url';
import path from 'node:path';
import {openTeamZip,sha256} from './team-update-zip.mjs';

const ROOT=fileURLToPath(new URL('../../../',import.meta.url));
const BASELINE=new URL('./team-source-baseline.json',import.meta.url);
const TEAM_ANCHOR='modules/analytics/contract.ts';
const PROBE_ANCHOR='packs/BP/scripts/modules/analytics/contract.ts';
const MAX_MODULE_FILES=160;
const MAX_ISSUES=16;
const SHA=/^[a-f0-9]{64}$/;
const MOD=/^[a-z][a-z0-9-]{0,39}$/;
const REL=/^[a-z0-9_.-]+(?:\/[a-zA-Z0-9_.-]+){1,8}$/;

function fail(reason){throw new Error(reason);}
function onlyInteger(value){return Number.isSafeInteger(value)&&value>=0;}
function rooted(entries,anchor){
  const paths=[...entries].filter(name=>name.endsWith(anchor));
  if(paths.length!==1)fail('AMBIGUOUS_ZIP_LAYOUT');
  const root=paths[0].slice(0,-anchor.length);
  if(root&&!/^[A-Za-z0-9._-]{1,100}\/$/.test(root))
    fail('AMBIGUOUS_ZIP_LAYOUT');
  return root;
}
function sourceFile(relative){
  return relative.endsWith('/module.json') ||
    (/\.(?:ts|js|mjs)$/.test(relative)&&
     !/\.(?:test|spec)\.(?:ts|js|mjs)$/.test(relative));
}
async function runtimeTree(zip,base){
  const names=[...zip.entries].filter(name=>name.startsWith(base))
    .map(name=>name.slice(base.length)).filter(sourceFile).sort();
  if(names.length<1||names.length>MAX_MODULE_FILES)
    fail('UNSUPPORTED_RUNTIME_MODULE_COUNT');
  const rows=new Map(),modules=new Map();
  for(const relative of names){
    if(!REL.test(relative)||!MOD.test(relative.split('/')[0]))
      fail('UNSUPPORTED_MODULE_FILENAME');
    const digest=sha256(await zip.readMember(base+relative));
    rows.set(relative,digest);
    const module=relative.split('/')[0];
    if(!modules.has(module))modules.set(module,[]);
    modules.get(module).push(relative+'\0'+digest+'\n');
  }
  const trees={};
  for(const name of [...modules.keys()].sort()){
    trees[name]=sha256(Buffer.from(modules.get(name).join(''),'utf8'));
  }
  return {rows,trees};
}
function parseJson(bytes,label){
  let parsed;
  try{parsed=JSON.parse(bytes.toString('utf8'));}
  catch{fail('INVALID_'+label);}
  if(!parsed||typeof parsed!=='object'||Array.isArray(parsed))
    fail('INVALID_'+label);
  return parsed;
}
function matchConst(text,name) {
  const expression=new RegExp(
    '^export const '+name+' = "([^"\\r\\n]{1,90})";\\s*$','gm');
  const found=[...text.matchAll(expression)];
  if(found.length!==1)fail('UNRECOGNIZED_TEAM_CONTRACT');
  return found[0][1];
}
function matchNumber(text,name){
  const expression=new RegExp(
    '^export const '+name+' = (\\d+)(?: \\* (\\d+))?;\\s*$','gm');
  const found=[...text.matchAll(expression)];
  if(found.length!==1)fail('UNRECOGNIZED_TEAM_CONTRACT');
  const value=Number(found[0][1])*(found[0][2]?Number(found[0][2]):1);
  if(!onlyInteger(value)||value>256*1024)fail('UNRECOGNIZED_TEAM_CONTRACT');
  return value;
}
function scoreTypes(text){
  const match=text.match(/^export const SCORED_RECORD_TYPES: readonly string\[\] = (\[[^\r\n]{1,300}\]);$/m);
  if(!match)fail('UNRECOGNIZED_TEAM_CONTRACT');
  const list=match[1].match(/"([a-z_]{2,40})"/g)?.map(v=>v.slice(1,-1))??[];
  if(list.length<1||list.length>8||new Set(list).size!==list.length)
    fail('UNRECOGNIZED_TEAM_CONTRACT');
  return list;
}
export function extractTeamWire(analytics,results){
  const a=analytics.toString('utf8');
  const g=results.toString('utf8');
  return {
    analytics:{schema:1,
      eventId:matchConst(a,'ANALYTICS_EVENT_ID'),
      marker:matchConst(a,'ANALYTICS_MARKER'),
      maxLineBytes:matchNumber(a,'ANALYTICS_MAX_LINE_BYTES'),
      linesPerTick:matchNumber(a,'ANALYTICS_LINES_PER_TICK'),
    },
    gameResults:{schema:1,
      eventId:matchConst(g,'GAME_RESULTS_EVENT_ID'),
      marker:matchConst(g,'GAME_RESULTS_MARKER'),
      maxLineBytes:matchNumber(g,'GAME_RESULTS_MAX_LINE_BYTES'),
      scoredTypes:scoreTypes(g),
    },
    probe:{commands:['probe:list','probe:run','probe:report']},
  };
}
function stableTreeDiff(pinned,current){
  return [...new Set([...Object.keys(pinned),...Object.keys(current)])]
    .filter(name=>pinned[name]!==current[name]).sort();
}
function immutableTopLevel(value){
  return value&&typeof value==='object'&&!Array.isArray(value);
}
function readProbeRuntime(manifest){
  if(!immutableTopLevel(manifest)||!immutableTopLevel(manifest.header)||
     !Array.isArray(manifest.header.min_engine_version)||
     manifest.header.min_engine_version.length!==3||
     manifest.header.min_engine_version.some(x=>!onlyInteger(x)||x>1000)||
     !Array.isArray(manifest.dependencies)||manifest.dependencies.length>30){
    fail('UNRECOGNIZED_EDUCATION_PROBE_TARGET');
  }
  const server=manifest.dependencies.filter(x=>
    immutableTopLevel(x)&&x.module_name==='@minecraft/server');
  if(server.length!==1||typeof server[0].version!=='string'||
     !/^[1-9]\d?\.\d{1,3}\.\d{1,3}$/.test(server[0].version)){
    fail('UNRECOGNIZED_EDUCATION_PROBE_TARGET');
  }
  return {minEngine:manifest.header.min_engine_version,
    serverModule:server[0].version};
}
function reportModules(list){
  return {count:list.length,modules:list.slice(0,12),
    omitted:Math.max(0,list.length-12)};
}
async function verifyProbeLocks(probe,root,moduleRepo,teamModuleTrees,probeSnapshot){
  const modulesLock=parseJson(
    await probe.readMember(root+'modules-lock.json'),'MODULES_LOCK');
  const skillsLock=parseJson(
    await probe.readMember(root+'skills-lock.json'),'SKILLS_LOCK');
  if(!immutableTopLevel(modulesLock.modules)||
     Object.keys(modulesLock.modules).length>40||
     typeof skillsLock.tag!=='string'||
     !/^v\d{1,3}\.\d{1,3}\.\d{1,3}$/.test(skillsLock.tag)||
     typeof skillsLock.commit!=='string'||
     !/^[a-f0-9]{40}$/.test(skillsLock.commit)||
     skillsLock.repo!==moduleRepo)fail('INVALID_TEAM_PROBE_LOCK');
  let checked=0;
  const lockProblems=[];
  const teamInstalls=[];
  // Every runtime module in the probe must be represented by its lock,
  // even if its upstream provider is an external dependency.
  for(const module of Object.keys(probeSnapshot.trees)){
    if(!Object.hasOwn(modulesLock.modules,module))lockProblems.push(module);
  }
  for(const [module,definition] of Object.entries(modulesLock.modules)){
    if(!MOD.test(module)||!immutableTopLevel(definition)||
       !immutableTopLevel(definition.files)||
       Object.keys(definition.files).length>50)fail('INVALID_TEAM_PROBE_LOCK');
    const teamOwned=definition.repo===moduleRepo;
    // A module with the same canonical team name must not silently
    // switch its lock provider to a foreign repository.
    if(!teamOwned&&Object.hasOwn(teamModuleTrees,module))
      lockProblems.push(module);
    if(teamOwned) {
      teamInstalls.push(module);
      if(definition.tag!==skillsLock.tag||
         definition.commit!==skillsLock.commit) {
        lockProblems.push(module);
      }
    }
    const lockedFiles=new Set(Object.keys(definition.files));
    const actualFiles=[...probe.entries].filter(name=>
      name.startsWith(root+'packs/BP/scripts/modules/'+module+'/'));
    if(actualFiles.some(name=>!lockedFiles.has(name.slice(root.length)))||
       lockedFiles.size!==actualFiles.length)lockProblems.push(module);
    for(const [filename,digest] of Object.entries(definition.files)){
      if(typeof filename!=='string'||
         !filename.startsWith('packs/BP/scripts/modules/'+module+'/')||
         !SHA.test(digest))fail('INVALID_TEAM_PROBE_LOCK');
      const file=root+filename;
      if(!probe.entries.has(file)) {
        lockProblems.push(module);continue;
      }
      checked++;
      if(sha256(await probe.readMember(file))!==digest)
        lockProblems.push(module);
    }
  }
  return {skillsLock:{tag:skillsLock.tag,commit:skillsLock.commit},
    checkedLockFiles:checked,
    teamInstalled:teamInstalls.sort(),
    problems:[...new Set(lockProblems)].sort()};
}
function boundedChanges(lockFiles,current){
  const changed=[];
  for(const [name,expected] of Object.entries(lockFiles)){
    if(current[name]!==expected)changed.push(name);
  }
  return changed.sort().slice(0,MAX_ISSUES);
}
function flagsFor(proposed,base,lock,parity){
  const contractChanged=JSON.stringify(proposed.wire)!==JSON.stringify(base.wire);
  const teamChanged=stableTreeDiff(base.runtimeTrees.team,proposed.runtimeTrees.team);
  const probeChanged=stableTreeDiff(base.runtimeTrees.probe,proposed.runtimeTrees.probe);
  const sourceDiff=boundedChanges(base.teamRequiredHashes,proposed.teamRequiredHashes);
  const probeDiff=boundedChanges(base.probeRequiredHashes,proposed.probeRequiredHashes);
  const educationTargetChanged=JSON.stringify(proposed.probeRuntime)!==
    JSON.stringify(base.probeRuntime);
  const archiveChanged=base.provenance.teamArchiveSha256!==
      proposed.provenance.teamArchiveSha256||
    base.provenance.probeArchiveSha256!==proposed.provenance.probeArchiveSha256;
  const criticalContractsChanged=sourceDiff.some(name=>
    name==='analytics/contract.ts'||
    name==='game-results/game-results-contract.ts');
  let status='PINNED_SNAPSHOTS_MATCH';
  if(contractChanged)status='BLOCKED_WIRE_CONTRACT_CHANGE';
  else if(educationTargetChanged)status='BLOCKED_EDUCATION_API_CHANGE';
  else if(criticalContractsChanged)status='BLOCKED_CONTRACT_SOURCE_REVIEW';
  else if(lock.problems.length>0)status='BLOCKED_PROBE_MODULE_LOCK';
  else if(parity.length>0)status='BLOCKED_TEAM_PROBE_DESYNC';
  else if(archiveChanged||teamChanged.length||probeChanged.length||
    sourceDiff.length||probeDiff.length)status='REVIEW_REQUIRED';
  return {status,contractChanged,educationTargetChanged,
    teamChanged,probeChanged,sourceDiff,probeDiff,archiveChanged};
}
export async function inspectTeamSourceZips({teamPath,probePath},baseline){
  if(!immutableTopLevel(baseline)||baseline.schema!==1||
     baseline.reviewState!=='APPROVED_SOURCE_REVIEW'||
     !immutableTopLevel(baseline.provenance)||
     !immutableTopLevel(baseline.wire)||
     !immutableTopLevel(baseline.teamRequiredHashes)||
     !immutableTopLevel(baseline.probeRequiredHashes)||
     !immutableTopLevel(baseline.probeRuntime)||
     !immutableTopLevel(baseline.runtimeTrees))fail('INVALID_LOCAL_SOURCE_BASELINE');
  const team=await openTeamZip(teamPath);
  let probe;
  try{
    probe=await openTeamZip(probePath);
    const teamRoot=rooted(team.entries,TEAM_ANCHOR);
    const probeRoot=rooted(probe.entries,PROBE_ANCHOR);
    const teamSnapshot=await runtimeTree(team,teamRoot+'modules/');
    const probeSnapshot=await runtimeTree(probe,probeRoot+
      'packs/BP/scripts/modules/');
    const wire=extractTeamWire(
      await team.readMember(teamRoot+'modules/analytics/contract.ts'),
      await team.readMember(teamRoot+'modules/game-results/game-results-contract.ts'));
    const probeRuntime=readProbeRuntime(parseJson(
      await probe.readMember(probeRoot+'packs/BP/manifest.json'),
      'PROBE_MANIFEST'));
    const runner=(await probe.readMember(probeRoot+
      'packs/BP/scripts/probe/runner/runner.ts')).toString('utf8');
    const runnerCommands=['probe:list','probe:run','probe:report'];
    const commandMissing=runnerCommands.filter(command=>
      !runner.includes(command));
    if(commandMissing.length)fail('TEAM_PROBE_RUNNER_COMMAND_CHANGE');
    const locks=await verifyProbeLocks(probe,probeRoot,
      baseline.provenance.moduleRepo,teamSnapshot.trees,probeSnapshot);
    const parity=locks.teamInstalled.filter(name=>
      !teamSnapshot.trees[name]||
      teamSnapshot.trees[name]!==probeSnapshot.trees[name]);
    const teamArchiveSha256=await team.archiveDigest();
    const probeArchiveSha256=await probe.archiveDigest();
    const requiredTeam={},requiredProbe={};
    for(const name of Object.keys(baseline.teamRequiredHashes)){
      requiredTeam[name]=sha256(
        await team.readMember(teamRoot+'modules/'+name));
    }
    for(const name of Object.keys(baseline.probeRequiredHashes)){
      requiredProbe[name]=sha256(
        await probe.readMember(probeRoot+name));
    }
    await team.verifyStable();
    await probe.verifyStable();
    const proposed={
      schema:1,purpose:baseline.purpose,
      provenance:{...baseline.provenance,
        teamArchiveSha256,probeArchiveSha256,
        upstreamTag:locks.skillsLock.tag,
        upstreamCommit:locks.skillsLock.commit},
      wire,teamRequiredHashes:requiredTeam,
      probeRequiredHashes:requiredProbe,probeRuntime,
      runtimeTrees:{team:teamSnapshot.trees,probe:probeSnapshot.trees},
    };
    const changes=flagsFor(proposed,baseline,locks,parity);
    // Do not call compatibility "PASS" just because two snapshots match:
    // source identity is not Education runtime or a game-specific test.
    const report={
      schema:1,status:changes.status,
      proof:'UNTRUSTED_OPERATOR_ZIP_OFFLINE_SOURCE_REVIEW_ONLY',
      compatibility:{
        wire:changes.contractChanged?'CHANGED_REVIEW_REQUIRED':'PINNED_WIRE_MATCH',
        educationTarget:changes.educationTargetChanged?
          'CHANGED_REVIEW_REQUIRED':'PINNED_BUILD_TARGET_MATCH',
        moduleLock:locks.problems.length?'INCONSISTENT':'SELF_CONSISTENT',
        installedTeamModules:parity.length?'DESYNCHRONIZED':'BYTE_MATCHED',
        educationRuntime:'NOT_VERIFIED',
      },
      candidate:{teamTag:locks.skillsLock.tag,
        teamCommit:locks.skillsLock.commit,
        checkedLockFiles:locks.checkedLockFiles,
        teamOwnedModulesInstalled:locks.teamInstalled.length},
      archives:{teamSha256:teamArchiveSha256,probeSha256:probeArchiveSha256},
      changes:{
        teamRuntimeModules:reportModules(changes.teamChanged),
        probeRuntimeModules:reportModules(changes.probeChanged),
        monitoredTeamFiles:reportModules(changes.sourceDiff),
        monitoredProbeFiles:reportModules(changes.probeDiff),
        divergentModules:reportModules(parity),
        badModuleLocks:reportModules(locks.problems),
      },
      reviewRequired:changes.status!=='PINNED_SNAPSHOTS_MATCH',
      updatesApplied:false,teamSourcesCopied:false,
      codexUpload:false,
      next:changes.status==='PINNED_SNAPSHOTS_MATCH'?
        'SOURCE_PIN_MATCH_RUNTIME_STILL_NOT_VERIFIED':
        'REVIEW_UPSTREAM_DIFF_AND_TEAM_LOCKS_BEFORE_ANY_APPROVAL',
    };
    // Only the operator reviewing an internally consistent changed snapshot
    // may manually copy this metadata into a future source commit.
    if(changes.status==='REVIEW_REQUIRED'){
      report.proposedBaseline={...proposed,
        reviewState:'CANDIDATE_NOT_APPROVED'};
    }
    return report;
  }finally{
    await team.close();
    if(probe)await probe.close();
  }
}
async function ordinary(dir){
  const st=await lstat(dir);
  if(!st.isDirectory()||st.isSymbolicLink())
    fail('UNSAFE_LOCAL_REVIEW_OUTPUT');
}
async function child(dir,name){
  await ordinary(dir);
  const dest=path.join(dir,name);
  try{await mkdir(dest,{mode:0o700});}
  catch(e){if(e.code!=='EEXIST')throw e;}
  await ordinary(dest);
  return dest;
}
async function archiveReport(report){
  const workspace=path.join(ROOT,'workspace');
  try{await mkdir(workspace,{mode:0o700});}
  catch(e){if(e.code!=='EEXIST')throw e;}
  const folder=await child(await child(workspace,'diagnostics'),'team-updates');
  const content=JSON.stringify(report,null,2)+'\n';
  // Re-reviewing the same ZIP against a newly approved baseline produces
  // a distinct report. Never overwrite an older review.
  const id=sha256(Buffer.from(content,'utf8')).slice(0,24);
  const file=path.join(folder,'review-'+id+'.json');
  if(Buffer.byteLength(content,'utf8')>24*1024)
    fail('LOCAL_REVIEW_EXCEEDS_BUDGET');
  try{
    const handle=await open(file,'wx',0o600);
    try{await handle.writeFile(content,'utf8');await handle.sync();}
    finally{await handle.close();}
  }catch(e){
    if(e.code!=='EEXIST')throw e;
    const metadata=await lstat(file,{bigint:true});
    if(!metadata.isFile()||metadata.isSymbolicLink()||
       metadata.nlink!==1n||metadata.size>24n*1024n)
      fail('LOCAL_REVIEW_CHANGED_DO_NOT_OVERWRITE');
    const existing=await readFile(file);
    if(existing.toString('utf8')!==content)
      fail('LOCAL_REVIEW_CHANGED_DO_NOT_OVERWRITE');
  }
  return 'workspace/diagnostics/team-updates/review-'+id+'.json';
}
export async function reviewTeamUpdates({teamPath,probePath,baseline}){
  const pin=baseline??parseJson(await readFile(BASELINE),'BASELINE');
  const result=await inspectTeamSourceZips({teamPath,probePath},pin);
  return {...result,reportFile:await archiveReport(result)};
}
if(process.argv[1]&&path.resolve(process.argv[1])===
   fileURLToPath(import.meta.url)){
  const args=process.argv.slice(2);
  if(args.length!==5||args[0]!=='inspect'||args[1]!=='--team'||
     args[3]!=='--probe'){
    console.error('Usage: node team-update.mjs inspect --team <absolute-team.zip> --probe <absolute-probe.zip>');
    process.exitCode=2;
  }else{
    reviewTeamUpdates({teamPath:args[2],probePath:args[4]})
      .then(report=>console.log(JSON.stringify({
        schema:1,status:report.status,
        compatibility:report.compatibility,
        changes:report.changes,reviewRequired:report.reviewRequired,
        reportFile:report.reportFile,
        updatesApplied:false,codexUpload:false,
      })))
      .catch(()=>{console.error('Team ZIP inspection refused; no update applied');
        process.exitCode=1;});
  }
}
