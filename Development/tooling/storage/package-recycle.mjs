// Post-success installer candidate retention, NEVER a Cargo target or user-data cleaner.
// Called ONLY by the verified native package owner after its complete build/stage path.
import {createHash} from 'node:crypto';
import {closeSync, fstatSync, lstatSync, openSync, opendirSync, readSync, rmdirSync, unlinkSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {planRetention} from './retention.mjs';

const PARENT = 'workspace/packages/lazy-tester';
const NAME = /^(\d+\.\d+\.\d+)-([a-f0-9]{12})-(\d{8}T\d{9}Z)$/;
const SHA256 = /^[0-9a-f]{64}$/;
const KEEP = 2;
const MAX_PRUNE_PER_BUILD = 8;
const HASH_CHUNK = 1024 * 1024;
const MAX_INSTALLER = 512 * 1024 * 1024;

function ordinaryDirectory(path) {
  const stat=lstatSync(path);
  if(!stat.isDirectory() || stat.isSymbolicLink())throw Error('Retention directory is linked or not ordinary');
  return stat;
}
function guardedParent(root) {
  const checkout=resolve(root);
  ordinaryDirectory(checkout);
  let current=checkout;
  for(const segment of PARENT.split('/')) {
    current=join(current,segment);
    ordinaryDirectory(current);
  }
  return current;
}
function sameDirectory(a,b) {
  return a.dev===b.dev && a.ino===b.ino && b.isDirectory() && !b.isSymbolicLink();
}
function unchanged(a,b) {
  return a.dev===b.dev && a.ino===b.ino && a.size===b.size && a.nlink===b.nlink &&
    a.mtimeMs===b.mtimeMs && a.ctimeMs===b.ctimeMs &&
    a.isFile()===b.isFile() && a.isDirectory()===b.isDirectory();
}
function ordinaryFile(path,max) {
  const stat=lstatSync(path);
  if(!stat.isFile() || stat.isSymbolicLink() || stat.nlink!==1 || stat.size<1 || stat.size>max){
    throw Error('Untrusted installer receipt or payload file');
  }
  return stat;
}
function boundedRead(path,max) {
  const before=ordinaryFile(path,max);
  const fd=openSync(path,'r');
  try {
    if(!unchanged(before,fstatSync(fd)))throw Error('Receipt file identity changed');
    const buffer=Buffer.alloc(before.size);
    let offset=0;
    while(offset<buffer.length){
      const n=readSync(fd,buffer,offset,buffer.length-offset,offset);
      if(!n)throw Error('Incomplete receipt');
      offset+=n;
    }
    if(!unchanged(before,lstatSync(path)))throw Error('Receipt changed during reading');
    return buffer.toString('utf8');
  }finally{closeSync(fd);}
}
function digestFile(path,expectedSize) {
  const before=ordinaryFile(path,MAX_INSTALLER);
  if(before.size!==expectedSize || before.size<64)throw Error('Installer byte count changed');
  const fd=openSync(path,'r'), hash=createHash('sha256'), buffer=Buffer.alloc(HASH_CHUNK);
  try {
    if(!unchanged(before,fstatSync(fd)))throw Error('Installer identity changed during read');
    let offset=0;
    while(offset<before.size){
      const n=readSync(fd,buffer,0,Math.min(buffer.length,before.size-offset),offset);
      if(!n)throw Error('Incomplete installer hash');
      hash.update(buffer.subarray(0,n));
      offset+=n;
    }
  }finally{closeSync(fd);}
  if(!unchanged(before,lstatSync(path)))throw Error('Installer metadata changed during hash');
  return {hash:hash.digest('hex'),stat:before};
}
function exactContents(folder,names){
  const handle=opendirSync(folder),found=new Set();
  try {
    let entry;
    while((entry=handle.readSync())!==null){
      if(!names.has(entry.name) || found.has(entry.name))throw Error('Candidate contains unknown or duplicate entries');
      found.add(entry.name);
    }
  }finally{handle.closeSync();}
  if(found.size!==names.size)throw Error('Candidate missing receipt or installer');
}
function validateCandidate(parent,name) {
  const match=NAME.exec(name);
  if(!match)throw Error('Candidate directory is not a known package export');
  const version=match[1],folder=join(parent,name),stat=ordinaryDirectory(folder);
  const installer=`Lazy-Tester-${version}-windows-x86_64-setup.exe`;
  const files=['package-manifest.json','SHA256SUMS.txt',installer];
  exactContents(folder,new Set(files));
  const manifest=JSON.parse(boundedRead(join(folder,files[0]),4096));
  const sums=boundedRead(join(folder,files[1]),256);
  const checksum=manifest?.installer?.sha256;
  if(manifest?.schema!==1 || manifest?.product!=='Lazy Tester' ||
     manifest.version!==version || manifest.sourceBranch!=='Local' ||
     typeof manifest.sourceCommit!=='string' || !/^[0-9a-f]{40}$/.test(manifest.sourceCommit) ||
     manifest.sourceCommit.slice(0,12)!==match[2] ||
     manifest.classification!=='DEVELOPMENT_CANDIDATE' || manifest.releaseReady!==false ||
     manifest.publication!=='DISABLED' || manifest.signing!=='NOT_VERIFIED' ||
     manifest.installedAcceptance!=='NOT_EXECUTED' || manifest.minecraftAcceptance!=='NOT_EXECUTED' ||
     manifest.installer.name!==installer || !SHA256.test(checksum??'') ||
     !Number.isSafeInteger(manifest.installer.bytes) ||
     sums!==`${checksum}  ${installer}\n`){
    throw Error('Candidate metadata is inconsistent');
  }
  const actual=digestFile(join(folder,installer),manifest.installer.bytes);
  if(actual.hash!==checksum)throw Error('Installer SHA-256 does not match verified receipt');
  return {folder,name,stat,files:files.map(file=>({name:file,
    stat:file===installer?actual.stat:ordinaryFile(join(folder,file),file===files[0]?4096:256)}))};
}
function unlinkExact(parent,candidate) {
  const folder=join(parent,candidate.name),current=ordinaryDirectory(folder);
  if(!sameDirectory(candidate.stat,current))throw Error('Package candidate directory changed before deletion');
  exactContents(folder,new Set(candidate.files.map(x=>x.name)));
  // Revalidate every ancestor and identity just before each exact-file unlink.
  for(const file of candidate.files){
    ordinaryDirectory(parent);
    if(!sameDirectory(candidate.stat,ordinaryDirectory(folder)))throw Error('Candidate directory replaced during retention');
    const absolute=join(folder,file.name);
    if(!unchanged(file.stat,ordinaryFile(absolute,file.stat.size))){
      throw Error('Candidate entry changed before deletion');
    }
    unlinkSync(absolute);
  }
  if(!sameDirectory(candidate.stat,ordinaryDirectory(folder)))throw Error('Candidate directory replaced after deletion');
  rmdirSync(folder); // Only the now-empty exact receipt-owned folder.
}

// Protected: latest 2 verified candidates, App/Data, private backups, active Cargo target,
// compiler archive, toolchains and any unknown or linked directory/receipt.
export function pruneSupersededCandidates(root,currentCandidateName,options={}) {
  if(typeof currentCandidateName!=='string' || !NAME.test(currentCandidateName)){
    throw Error('A newly exported, exactly named candidate is required');
  }
  const maxDeletes=options.maxDeletes??MAX_PRUNE_PER_BUILD;
  if(!Number.isInteger(maxDeletes) || maxDeletes<1 || maxDeletes>MAX_PRUNE_PER_BUILD){
    throw Error('Invalid bounded deletion limit');
  }
  const plan=(options.planner??planRetention)(root,{
    nowMs:options.nowMs??Date.now(),keepNewest:KEEP,minAgeDays:0,
    archiveMaxEntries:1,archiveDurationMs:1,
  });
  if(plan.packageStatus!=='OK')return {status:'BLOCKED',deleted:[],reason:'Incomplete or untrusted package inventory'};
  const rows=[...plan.packageRows].sort((a,b)=>b.time-a.time);
  const current=rows.find(row=>row.name===currentCandidateName);
  if(!current || current.status!=='PRESERVE' || rows[0]?.name!==currentCandidateName ||
     !Number.isFinite(current.time) || current.time> (options.nowMs??Date.now())+60_000){
    return {status:'BLOCKED',deleted:[],reason:'Current candidate identity/timestamp is not newest and trusted'};
  }
  const candidates=rows.filter(row=>row.status==='REVIEW_ONLY').reverse().slice(0,maxDeletes);
  if(candidates.length===0)return {status:'NO_CHANGE',deleted:[],remaining:0};
  const deleted=[];
  try {
    const parent=guardedParent(root);
    // Hash the newly built candidate first: do not discard older recoverable
    // binaries if the just-created latest export has been tampered with.
    validateCandidate(parent,currentCandidateName);
    const validated=candidates.map(row=>validateCandidate(parent,row.name));
    // Prepare every candidate before making any mutations.
    for(const item of validated){
      guardedParent(root);
      unlinkExact(parent,item);
      deleted.push(item.name);
    }
    return {status:'PRUNED',deleted,remaining:rows.filter(row=>row.status==='REVIEW_ONLY').length-deleted.length};
  }catch{
    return {status:deleted.length?'PARTIAL':'BLOCKED',deleted,
      reason:'Candidate hash/path changed or deletion could not be verified; inspect retention manually'};
  }
}
