// Read-only projection of CURRENT Local ancestry and existing commit trailers.
// Git commit metadata is historical author evidence, never an active policy,
// authorization, source execution receipt or Minecraft acceptance.
import {execFileSync} from 'node:child_process';
import {resolve} from 'node:path';

const SHA=/^[a-f0-9]{40}$/;
const VALID_STATES=new Set(['ACTIVE','BLOCKED','PAUSED','DONE']);
const TRAILER=/^(Work|State|Decision|Proof|Unresolved|Next):[ \t]*(.+)$/;
const MAX_HISTORY=120;
const MAX_OUTPUT=2*1024*1024;
const MAX_FIELD=1400;
export const HISTORY_CEILING='GIT_AUTHOR_RECORDED_CLAIM_NOT_CURRENT_SOURCE_OR_RUNTIME_PROOF';

function publicText(value,max=MAX_FIELD){
  return typeof value==='string'
    ?value.trim().replace(/[\x00-\x1f\x7f]/g,' ').slice(0,max)
    :'';
}
export function parseDecisionTrailers(body){
  if(typeof body!=='string'||body.length>20000)return undefined;
  const lines=body.replace(/\r\n/g,'\n').trimEnd().split('\n');
  // Require the conventional footer stanza; arbitrary "Work:" text in
  // a commit description is not a machine-recorded decision trailer.
  let start=lines.length-1;
  while(start>=0&&TRAILER.test(lines[start].trim()))start--;
  const stanza=lines.slice(start+1);
  if(stanza.length<3||start>=0&&lines[start].trim()!=='')return undefined;
  const values={};
  for(const line of stanza){
    const m=line.trim().match(TRAILER);
    if(!m||Object.hasOwn(values,m[1]))return undefined;
    values[m[1]]=publicText(m[2]);
    if(!values[m[1]])return undefined;
  }
  if(!values.Work||!values.State||!values.Proof||!VALID_STATES.has(values.State))
    return undefined;
  return {
    work:values.Work,state:values.State,decision:values.Decision??null,
    proofClaim:values.Proof,unresolved:values.Unresolved??null,
    nextClaim:values.Next??null,
  };
}

export function parseGitLog(raw){
  if(typeof raw!=='string'||raw.length>MAX_OUTPUT)throw new Error('BOUNDED_GIT_LOG_REQUIRED');
  const entries=[],issues=[];
  for(const chunk of raw.split('\x1f')){
    const line=chunk.replace(/^\s*\n/,'').trim();
    if(!line)continue;
    const parts=line.split('\x1e');
    if(parts.length!==3||!SHA.test(parts[0])||!/^\d{1,12}$/.test(parts[1])){
      issues.push('UNSUPPORTED_COMMIT_RECORD');continue;
    }
    const trailers=parseDecisionTrailers(parts[2]);
    if(!trailers)continue; // Unstructured history is not authoritative.
    entries.push({sha:parts[0],unixTime:Number(parts[1]),...trailers});
  }
  return {entries,issues};
}

export function gitReadOnly(root,args){
  return execFileSync('git',args,{
    cwd:resolve(root),encoding:'utf8',windowsHide:true,
    timeout:8000,maxBuffer:MAX_OUTPUT,
    stdio:['ignore','pipe','pipe'],
  });
}
function protectedGitPath(path){
  return !/^(?:Data|App|workspace|\.git)(?:\/|$)/i.test(path)&&
    !/(?:credential|password|token|secret|private.key)/i.test(path);
}
function limitedChangedFiles(root,sha,gitRead){
  const output=gitRead(root,['diff-tree','--root','--no-commit-id',
    '--no-renames','--name-only','-r',sha]);
  const paths=output.split(/\r?\n/).map(v=>v.trim()).filter(Boolean);
  const safe=paths.filter(p=>p.length<=512&&/^[A-Za-z0-9._/-]+$/.test(p)&&
    !p.startsWith('/')&&!p.startsWith('../')&&protectedGitPath(p));
  return {paths:safe.slice(0,100),hasOmittedPaths:safe.length>100||
    paths.length!==safe.length};
}

// Uses a bounded local repository when explicitly requested. No Git fetch,
// checkout, remote branch selection, worktree write or commit mutation.
// An injected read-only Git adapter allows deterministic nonexecuted fixtures.
export function readDecisionHistory(root,{
  work,sourcePath,limit=8,maxCommits=MAX_HISTORY,gitRead=gitReadOnly,
}={}){
  if(!Number.isInteger(limit)||limit<1||limit>12||
    !Number.isInteger(maxCommits)||maxCommits<1||maxCommits>MAX_HISTORY)
    throw new Error('Invalid bounded Git history scope');
  if((work===undefined)===(sourcePath===undefined))
    throw new Error('Select exactly one work ID or existing indexed source path');
  if(work!==undefined&&
    (typeof work!=='string'||work.length>180||!/^[A-Za-z0-9][A-Za-z0-9._ -]*$/.test(work)))
    throw new Error('Invalid exact Work trailer');
  if(sourcePath!==undefined&&
    (typeof sourcePath!=='string'||sourcePath.length>512||
     !/^[A-Za-z0-9._/-]+$/.test(sourcePath)||sourcePath.startsWith('/')||
     sourcePath.split('/').includes('..')||!protectedGitPath(sourcePath)))
    throw new Error('Invalid source-history path');
  let currentBranch,head,raw;
  try{
    currentBranch=gitRead(root,['symbolic-ref','--quiet','--short','HEAD']).trim();
    head=gitRead(root,['rev-parse','--verify','HEAD']).trim();
    if(currentBranch!=='Local'||!SHA.test(head))
      return {status:'LOCAL_HEAD_UNAVAILABLE',branch:currentBranch||'UNKNOWN',
        proofBoundary:HISTORY_CEILING,commits:[]};
    const args=['log','--no-show-signature','--no-patch',
      '--max-count='+maxCommits,
      '--format=%H%x1e%ct%x1e%B%x1f','HEAD'];
    if(sourcePath)args.push('--',sourcePath);
    raw=gitRead(root,args);
  }catch{
    return {status:'GIT_READ_UNAVAILABLE',proofBoundary:HISTORY_CEILING,
      commits:[],note:'No Git history was inspected; do not infer absence of decisions.'};
  }
  const parsed=parseGitLog(raw);
  const candidates=work===undefined?parsed.entries:
    parsed.entries.filter(x=>x.work===work);
  const selected=candidates.slice(0,limit);
  const commits=[],gaps=[...parsed.issues];
  for(const record of selected){
    let change;
    try{change=limitedChangedFiles(root,record.sha,gitRead);}
    catch{gaps.push('COMMIT_FILE_WITNESS_UNAVAILABLE:'+record.sha);continue;}
    if(sourcePath&&!change.paths.includes(sourcePath)){
      gaps.push('SCOPED_PATH_WITNESS_UNAVAILABLE:'+record.sha);continue;
    }
    commits.push({...record,
      relation:sourcePath?'EXACT_FILE_MODIFIED_AT_COMMIT':'EXACT_WORK_TRAILER_MATCH',
      changedPaths:change.paths,changedPathsTruncated:change.hasOmittedPaths,
      proofAuthority:'GIT_AUTHOR_CLAIM_ONLY'});
  }
  const countRaw=raw.split('\x1f').filter(x=>x.trim()).length;
  return {
    schemaVersion:1,mode:'GIT_DECISION_PROVENANCE',
    status:gaps.length?'PARTIAL_GIT_EVIDENCE':'HISTORY_BOUNDED',
    branch:'Local',head,
    query:sourcePath?{kind:'EXACT_SOURCE_PATH',value:sourcePath}:
      {kind:'EXACT_WORK_ID',value:work},
    scope:{maxCommits,limit,scanned:countRaw,
      mayHaveOlderHistory:countRaw>=maxCommits,
      omittedMatchingCommits:Math.max(0,candidates.length-limit),
      renamesNotFollowed:true},
    commits,gaps,
    proofBoundary:HISTORY_CEILING,
    note:'Commit Work, Decision, State and Proof are recorded author claims. DONE closes only that commit outcome. Git history never replaces current source, canonical policy, or execution evidence.',
  };
}
export function decisionHistoryMarkdown(result){
  const tick=String.fromCharCode(96);
  const lines=['# Git decision provenance','',
    'Status: **'+result.status+'**',
    'Evidence ceiling: '+tick+result.proofBoundary+tick,
    'Current branch/HEAD: '+tick+(result.branch??'UNKNOWN')+
      ' / '+(result.head??'UNVERIFIED')+tick,'',
    'A commit trailer is historical author testimony, not current authority or runtime proof.',''];
  for(const entry of result.commits){
    lines.push('## '+tick+entry.sha+tick,'',
      '- Work: '+tick+entry.work+tick,
      '- Commit state: '+tick+entry.state+tick,
      '- Recorded decision: '+(entry.decision??'NOT_RECORDED'),
      '- Recorded proof claim: '+entry.proofClaim,
      '- Unresolved at that commit: '+(entry.unresolved??'NOT_RECORDED'),
      '- Relation: '+entry.relation,
      '- Changed paths (bounded): '+entry.changedPaths.map(x=>tick+x+tick).join(', '),'');
  }
  if(!result.commits.length)lines.push('No verifiable matching decision in the bounded, accessible history. This is not evidence that none exists.','');
  if(result.scope?.mayHaveOlderHistory)
    lines.push('The history search reached its cap; older decisions may exist.','');
  for(const gap of result.gaps??[])lines.push('- Evidence gap: '+tick+gap+tick);
  return lines.join('\n')+'\n';
}
