// Read-only Cargo artifact breakdown. No file content, cache cleanup, or data/profile access.
import {lstatSync, opendirSync} from 'node:fs';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';

export const CARGO_ROOTS = Object.freeze([
  {name:'Active desktop Cargo target', path:'workspace/builds/desktop/target'},
  {name:'Legacy desktop Cargo target', path:'Development/apps/desktop/src-tauri/target'},
  {name:'Legacy runtime-core Cargo target', path:'Development/engine/runtime-core/target'},
]);
const WIN_TARGET = 'x86_64-pc-windows-msvc';
const COMPONENTS = new Set(['deps','incremental','build','.fingerprint','examples','bundle']);
const DEFAULT_MAX_ENTRIES = 500_000;
const DEFAULT_MAX_MS = 30_000;
const MAX_DEPTH = 64;

export function cargoBucket(parts) {
  let prefix='Other target or root', rest=parts;
  if(parts[0]===WIN_TARGET && ['debug','release'].includes(parts[1])) {
    prefix=`Windows MSVC ${parts[1]}`;
    rest=parts.slice(2);
  } else if(['debug','release'].includes(parts[0])) {
    prefix=`Host-default ${parts[0]}`;
    rest=parts.slice(1);
  }
  const component=COMPONENTS.has(rest[0]) ? rest[0] : 'other';
  return `${prefix} / ${component}`;
}

function safeRoot(root, relative) {
  let current=root;
  for(const part of relative.split('/')) {
    current=join(current,part);
    let stats;
    try {stats=lstatSync(current);}
    catch(error){return {status:error?.code==='ENOENT'?'ABSENT':'PARTIAL'};}
    if(stats.isSymbolicLink() || !stats.isDirectory())return {status:'PARTIAL'};
  }
  return {status:'OK', path:current};
}

function scanRoot(root, category, budget) {
  const result={name:category.name,path:category.path,status:'OK',bytes:0n,files:0,
    directories:0,links:0,hardlinkedFiles:0,errors:0,buckets:new Map()};
  if(budget.exhausted)return {...result,status:'PARTIAL'};
  const admission=safeRoot(root,category.path);
  if(admission.status!=='OK')return {...result,status:admission.status,errors:admission.status==='PARTIAL'?1:0};
  const stack=[{path:admission.path,parts:[],opened:false,handle:null}];
  try {
    while(stack.length){
      if(Date.now()>=budget.deadline || budget.visited>=budget.maxEntries){
        result.status='PARTIAL';budget.exhausted=true;break;
      }
      const node=stack[stack.length-1];
      if(!node.opened){
        node.opened=true;
        budget.visited++;
        let info;
        try{info=lstatSync(node.path,{bigint:true});}
        catch{result.errors++;result.status='PARTIAL';stack.pop();continue;}
        if(info.isSymbolicLink()){
          result.links++;result.status='PARTIAL';stack.pop();continue;
        }
        if(info.isFile()){
          const kind=cargoBucket(node.parts);
          const prior=result.buckets.get(kind)??{bytes:0n,files:0};
          prior.bytes+=info.size;prior.files++;
          result.buckets.set(kind,prior);
          result.bytes+=info.size;result.files++;
          if(info.nlink>1n)result.hardlinkedFiles++;
          stack.pop();continue;
        }
        if(!info.isDirectory()){
          result.errors++;result.status='PARTIAL';stack.pop();continue;
        }
        result.directories++;
        try{node.handle=opendirSync(node.path);}
        catch{result.errors++;result.status='PARTIAL';stack.pop();continue;}
      }
      let child;
      try{child=node.handle.readSync();}
      catch{result.errors++;result.status='PARTIAL';try{node.handle.closeSync();}catch{};node.handle=null;stack.pop();continue;}
      if(child===null){
        try{node.handle.closeSync();}
        catch{result.errors++;result.status='PARTIAL';}
        node.handle=null;stack.pop();continue;
      }
      if(stack.length>=MAX_DEPTH){result.errors++;result.status='PARTIAL';continue;}
      stack.push({path:join(node.path,child.name),parts:[...node.parts,child.name],opened:false,handle:null});
    }
  }finally{
    for(const node of stack)if(node.handle)try{node.handle.closeSync();}catch{result.errors++;result.status='PARTIAL';}
  }
  return result;
}

export function inspectCargoFootprint(root,options={}){
  const location=resolve(root);
  const stats=lstatSync(location);
  if(!stats.isDirectory() || stats.isSymbolicLink())throw Error('Checkout root must be an ordinary directory');
  const maxEntries=options.maxEntries??DEFAULT_MAX_ENTRIES;
  const maxDurationMs=options.maxDurationMs??DEFAULT_MAX_MS;
  if(!Number.isSafeInteger(maxEntries)||maxEntries<1||
     !Number.isSafeInteger(maxDurationMs)||maxDurationMs<1)throw Error('Invalid positive scan limits');
  const budget={maxEntries,visited:0,deadline:Date.now()+maxDurationMs,exhausted:false};
  const roots=options.roots??CARGO_ROOTS;
  const rows=roots.map(category=>scanRoot(location,category,budget));
  const total=rows.reduce((sum,row)=>sum+row.bytes,0n);
  return {rows,total,complete:rows.every(row=>row.status==='OK'||row.status==='ABSENT'),
    visited:budget.visited,limited:budget.exhausted};
}
const fmt=bytes=>`${(Number(bytes)/1048576).toFixed(2)} MiB`;
export function renderCargoFootprint(result){
  const out=['Lazy Tester Cargo footprint — read-only logical file sizes'];
  for(const root of result.rows){
    out.push(`${root.name}: ${fmt(root.bytes)} [${root.status}], files=${root.files}, hardlinks=${root.hardlinkedFiles}`);
    const all=[...root.buckets].sort((a,b)=>a[1].bytes===b[1].bytes?0:a[1].bytes>b[1].bytes?-1:1);
    for(const [kind,data] of all.slice(0,16))out.push(`  ${kind}: ${fmt(data.bytes)} (${data.files} files)`);
    if(all.length>16)out.push(`  Other ${all.length-16} groups: ${fmt(all.slice(16).reduce((s,[,v])=>s+v.bytes,0n))}`);
    if(root.status==='PARTIAL')out.push('  INCOMPLETE: permission, link or traversal limit prevented an exact count.');
    if(root.buckets.has('Host-default debug / deps')&&root.buckets.has('Windows MSVC debug / deps')){
      out.push('  OBSERVATION: host-default and explicit Windows debug outputs coexist; not proof of redundant bytes.');
    }
  }
  out.push(`Measured Cargo target total: ${fmt(result.total)}${result.complete?'':' (LOWER BOUND, partial)'}`);
  out.push(`Inspected entries: ${result.visited}${result.limited?' (limit reached)':''}.`);
  out.push('These are logical file sizes, NOT disk allocation or safe/reclaimable space.');
  out.push('No cleanup, file contents, package export, user Data, worlds or private backups were accessed.');
  return out.join('\n');
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  if(process.argv.length!==2){console.error('Usage: cargo-footprint.mjs (read-only; no --apply)');process.exitCode=2;}
  else try{
    const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));
    const report=inspectCargoFootprint(root);
    console.log(renderCargoFootprint(report));
    if(!report.complete)process.exitCode=3;
  }catch(error){console.error(`Cargo footprint scan failed: ${error?.code??error?.message??'unknown'}`);process.exitCode=1;}
}
