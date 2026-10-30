// Developer cache retention planning — READ ONLY. This tool never deletes files.
// Repository policy requires separate Windows evidence and approval before any prune.
import {closeSync, lstatSync, opendirSync, openSync, readSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {collectInventory, CATEGORIES} from './inventory.mjs';

const PACKAGE_PARENT = 'workspace/packages/lazy-tester';
const ARCHIVED_COMPILER = 'workspace/archive/compiler-cache-20261026';
const MAX_PACKAGE_DIRECTORIES = 300;
const MAX_ENTRIES = 1000;
const MAX_RECEIPT_BYTES = 4096;
const MAX_CHECKSUM_BYTES = 256;
const MAX_RUN_MS = 20000;
const KEEP_NEWEST = 3;
const MIN_AGE_DAYS = 30;
const DAY_MS = 24 * 60 * 60 * 1000;
const VERSION_DIR = /^(\d+\.\d+\.\d+)-([a-f0-9]{12})-(\d{8}T\d{9}Z)$/;
const SHA256 = /^[a-f0-9]{64}$/;
const SHA40 = /^[a-f0-9]{40}$/;
const FILES = ['package-manifest.json', 'SHA256SUMS.txt'];

function ordinaryChain(root, fragments) {
  let target=root;
  for (const fragment of fragments) {
    target=join(target,fragment);
    let stats;
    try { stats=lstatSync(target); }
    catch(error) { return {status:error?.code==='ENOENT'?'ABSENT':'BLOCKED'}; }
    if (!stats.isDirectory() || stats.isSymbolicLink()) return {status:'BLOCKED'};
  }
  return {status:'OK', target};
}
function boundedText(path, maximum) {
  const info=lstatSync(path);
  if (!info.isFile() || info.isSymbolicLink() || info.nlink!==1 || info.size<1 || info.size>maximum) {
    throw new Error('Receipt is not a bounded ordinary file');
  }
  // A read-only, fixed-name receipt only. Never inspect arbitrary payload content.
  const fd=openSync(path,'r');
  try {
    const check=lstatSync(path);
    if (!check.isFile() || check.isSymbolicLink() || check.nlink!==1 ||
        check.size!==info.size || check.size>maximum) throw new Error('Receipt changed');
    const buffer=Buffer.alloc(info.size);
    let offset=0;
    while(offset<buffer.length) {
      const read=readSync(fd,buffer,offset,buffer.length-offset,offset);
      if(read===0) throw new Error('Receipt is incomplete');
      offset+=read;
    }
    return buffer.toString('utf8');
  }finally{closeSync(fd);}
}
function stampTime(stamp) {
  const nums=[stamp.slice(0,4),stamp.slice(4,6),stamp.slice(6,8),
    stamp.slice(9,11),stamp.slice(11,13),stamp.slice(13,15),stamp.slice(15,18)].map(Number);
  const [y,mo,d,h,mi,s,ms]=nums;
  const time=Date.UTC(y,mo-1,d,h,mi,s,ms);
  const parsed=new Date(time);
  if (!Number.isFinite(time) || parsed.getUTCFullYear()!==y || parsed.getUTCMonth()+1!==mo ||
      parsed.getUTCDate()!==d || parsed.getUTCHours()!==h || parsed.getUTCMinutes()!==mi ||
      parsed.getUTCSeconds()!==s || parsed.getUTCMilliseconds()!==ms) return null;
  return time;
}
function checkCandidate(folder,name) {
  const m=VERSION_DIR.exec(name);
  const blocked={name,status:'BLOCKED',reason:'Unrecognized or incomplete candidate; preserve for manual review',bytes:0n};
  if(!m) return {...blocked,name:'[unrecognized candidate]'};
  const [,version,shortSha,stamp]=m;
  const time=stampTime(stamp);
  if(time===null) return {...blocked,reason:'Invalid UTC candidate stamp'};
  const candidate=join(folder,name);
  let rootInfo;
  try{rootInfo=lstatSync(candidate);}catch{return {...blocked,reason:'Candidate disappeared during scan'};}
  if (!rootInfo.isDirectory() || rootInfo.isSymbolicLink()) return {...blocked,reason:'Candidate is not an ordinary directory'};
  const installerName=`Lazy-Tester-${version}-windows-x86_64-setup.exe`;
  const expected=new Set([...FILES,installerName]);
  let dir;
  try{dir=opendirSync(candidate);}catch{return {...blocked,reason:'Candidate cannot be inspected'};}
  const found=new Set();
  try{
    let ent;
    while((ent=dir.readSync())!==null) {
      if(!expected.has(ent.name) || found.has(ent.name))return {...blocked,reason:'Unrecognized file in candidate'};
      found.add(ent.name);
    }
  }catch{return {...blocked,reason:'Candidate directory inspection failed'};}
  finally{try{dir.closeSync();}catch{}}
  if(found.size!==expected.size) return {...blocked,reason:'Incomplete three-file candidate'};
  let info;
  try{
    const file=join(candidate,installerName);
    info=lstatSync(file);
    if(!info.isFile() || info.isSymbolicLink() || info.nlink!==1 || info.size<64 ||
        info.size>512*1024*1024) throw new Error('Invalid installer metadata');
    const manifest=JSON.parse(boundedText(join(candidate,'package-manifest.json'),MAX_RECEIPT_BYTES));
    const hash=manifest?.installer?.sha256;
    const checksum=boundedText(join(candidate,'SHA256SUMS.txt'),MAX_CHECKSUM_BYTES);
    if(manifest?.schema!==1 || manifest?.product!=='Lazy Tester' || manifest.version!==version ||
      manifest.sourceBranch!=='Local' || !SHA40.test(manifest.sourceCommit??'') ||
      manifest.sourceCommit.slice(0,12)!==shortSha ||
      manifest.classification!=='DEVELOPMENT_CANDIDATE' || manifest.releaseReady!==false ||
      manifest.publication!=='DISABLED' || manifest.signing!=='NOT_VERIFIED' ||
      manifest.installedAcceptance!=='NOT_EXECUTED' || manifest.minecraftAcceptance!=='NOT_EXECUTED' ||
      manifest.installer.name!==installerName || !SHA256.test(hash??'') ||
      manifest.installer.bytes!==info.size || checksum!==`${hash}  ${installerName}\n`) {
      throw new Error('Candidate receipt does not match its fixed identity');
    }
    // Receipt agreement is not an installer SHA-256 verification.
    return {name,time,status:'PRESERVE',reason:'Retention policy not applied yet',
      bytes:BigInt(info.size)+BigInt(Buffer.byteLength(checksum))+
        BigInt(infoSize(join(candidate,'package-manifest.json')))};
  }catch{return {...blocked,reason:'Receipt or installer metadata is invalid'};}
}
function infoSize(path){return lstatSync(path).size;}

export function planRetention(root, options={}) {
  const location=resolve(root);
  const top=lstatSync(location);
  if(!top.isDirectory() || top.isSymbolicLink()) throw new Error('Checkout root must be ordinary');
  const now=options.nowMs ?? Date.now();
  const maxDirs=options.maxDirectories ?? MAX_PACKAGE_DIRECTORIES;
  const maxEntries=options.maxEntries ?? MAX_ENTRIES;
  const maxMs=options.maxDurationMs ?? MAX_RUN_MS;
  // The default storage report stays conservative (3 copies / 30 days).
  // Only a completed package export may request bounded latest-two turnover.
  const keepNewest=options.keepNewest ?? KEEP_NEWEST;
  const minAgeDays=options.minAgeDays ?? MIN_AGE_DAYS;
  if(!Number.isSafeInteger(keepNewest) || keepNewest<2 || keepNewest>KEEP_NEWEST ||
     !Number.isSafeInteger(minAgeDays) || minAgeDays<0 || minAgeDays>MIN_AGE_DAYS) {
    throw Error('Invalid guarded package retention policy');
  }
  if(!Number.isFinite(now) || !Number.isSafeInteger(maxDirs) || maxDirs<1 ||
     !Number.isSafeInteger(maxEntries) || maxEntries<1 ||
     !Number.isSafeInteger(maxMs) || maxMs<1) throw new Error('Invalid finite planner limits');
  const started=Date.now();
  const packageRoot=ordinaryChain(location,PACKAGE_PARENT.split('/'));
  const packages=[];
  let status=packageRoot.status;
  let truncated=false;
  if(status==='OK') {
    let dir;
    try{dir=opendirSync(packageRoot.target);}catch{status='BLOCKED';}
    if(dir){
      let scanned=0;
      try {
        let next;
        while((next=dir.readSync())!==null){
          if(scanned>=maxEntries || packages.length>=maxDirs || Date.now()-started>=maxMs){
            truncated=true;
            break;
          }
          scanned++;
          if(!next.isDirectory() || next.isSymbolicLink()){
            packages.push({name:'[unrecognized entry]',status:'BLOCKED',
              reason:'Non-directory or reparse entry at package root',bytes:0n});
            continue;
          }
          packages.push(checkCandidate(packageRoot.target,next.name));
        }
      }catch{truncated=true;}
      finally{try{dir.closeSync();}catch{truncated=true;}}
    }
  }
  if(truncated) status='PARTIAL';
  else if(status==='OK' && packages.some(p=>p.status==='BLOCKED')) status='BLOCKED';
  // Only complete discovery may rank candidates by newest. Never propose a
  // review for a potentially newest entry when discovery is incomplete.
  if(status==='OK') {
    const valid=packages.filter(p=>p.status==='PRESERVE').sort((a,b)=>b.time-a.time);
    valid.forEach((p,i)=>{
      const ageMs=now-p.time;
      if(ageMs<0 || i<keepNewest || ageMs<minAgeDays*DAY_MS){
        p.status='PRESERVE';
        p.reason=ageMs<0?'Future-dated candidate; preserve':i<keepNewest?'Protected latest candidate':`Less than ${minAgeDays} days old`;
      }else{
        p.status='REVIEW_ONLY';
        p.reason=minAgeDays===0
          ?`Outside newest ${keepNewest}; post-build authentication required`
          :`Older than ${minAgeDays} days and outside newest ${keepNewest}; explicit approval required`;
      }
    });
  }else{
    packages.forEach(p=>{if(p.status==='PRESERVE'){p.status='BLOCKED';p.reason='Untrusted or incomplete package inventory; cannot rank safely';}});
  }
  const archive=collectInventory(location,{categories:[CATEGORIES.find(c=>c.path===ARCHIVED_COMPILER)],
    maxDurationMs:options.archiveDurationMs ?? 5000,maxVisitedEntries:options.archiveMaxEntries ?? 25000}).rows[0];
  const archiveState=archive.state==='missing'?'ABSENT':archive.state==='partial'?'BLOCKED':'REVIEW_ONLY';
  return {packageStatus:status,packageRows:packages,archive:{name:'Archived compiler cache',
    status:archiveState,bytes:archive.bytes,reason:archiveState==='REVIEW_ONLY'
      ?'Historical compiler output; provenance and retention not yet approved'
      :archiveState==='BLOCKED'?'Incomplete, linked or inaccessible archive':'Archive not present'},
    complete:status!=='PARTIAL'&&status!=='BLOCKED'&&archiveState!=='BLOCKED',
    policy:{keepNewest,minAgeDays},
    destructiveCapability:false};
}

function formatBytes(bytes){return `${(Number(bytes)/(1024**2)).toFixed(2)} MiB`;}
export function renderPlan(plan) {
  const lines=[
    'Lazy Tester retention plan — DRY RUN ONLY / NO DELETE CAPABILITY',
    'Scope: historical compiler cache and versioned Lazy Tester candidate exports only.',
    `Policy: preserve newest ${plan.policy.keepNewest} valid packages and all candidates younger than ${plan.policy.minAgeDays} days.`,
    `Package inventory: ${plan.packageStatus}; candidates discovered: ${plan.packageRows.length}.`,
  ];
  for(const p of [...plan.packageRows].sort((a,b)=>a.name.localeCompare(b.name))){
    lines.push(`${p.status.padEnd(11)} ${p.name} (${formatBytes(p.bytes)}) — ${p.reason}`);
  }
  lines.push(`${plan.archive.status.padEnd(11)} ${plan.archive.name} (${formatBytes(plan.archive.bytes)}) — ${plan.archive.reason}`);
  lines.push('REVIEW_ONLY is not permission to delete. BLOCKED items must be preserved.');
  lines.push('Logical file sizes are not allocated bytes or recoverable disk space.');
  lines.push('Data, App, active Cargo target, toolchains, private backups and worlds are excluded.');
  lines.push('No data modified. No apply/delete/cleanup mode is implemented.');
  return lines.join('\n');
}

if(process.argv[1] && resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  if(process.argv.length!==2){
    console.error('Read-only usage: node tooling/storage/retention.mjs (no --apply mode)');
    process.exitCode=2;
  }else{
    try{
      const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));
      const plan=planRetention(root);
      console.log(renderPlan(plan));
      if(!plan.complete) process.exitCode=3;
    }catch(error){
      console.error('Retention plan blocked: '+(error?.code??error?.message??'unknown error'));
      process.exitCode=1;
    }
  }
}
