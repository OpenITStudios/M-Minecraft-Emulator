// Single-purpose, opt-in retirement of the OLD compiler archive only.
// No generic delete API, no automatic invocation from build or startup.
import {existsSync, lstatSync, opendirSync, readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {join, resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {inspectCargoFootprint, CARGO_ROOTS} from './cargo-footprint.mjs';

export const ARCHIVE = 'workspace/archive/compiler-cache-20261026/target';
export const APPROVED_ROOTS = Object.freeze([
  {name:'Archived old compiler target',path:ARCHIVE},
]);
const MAX_ENTRIES=750000;
const MAX_SCAN_MS=90000;
const ROOT_DIRECTORY_NAMES=new Set(['debug','release','x86_64-pc-windows-msvc','cargo-timings']);
const ROOT_FILE_NAMES=new Set(['CACHEDIR.TAG','.rustc_info.json']);

function ordinaryDirectory(path) {
  const s=lstatSync(path);
  if(!s.isDirectory() || s.isSymbolicLink())throw Error('Retired Cargo path must be an ordinary directory');
  return s;
}
function admitChain(root,relative) {
  let path=resolve(root);
  ordinaryDirectory(path);
  for(const part of relative.split('/')){
    path=join(path,part);
    if(!existsSync(path))return null;
    ordinaryDirectory(path);
  }
  return path;
}
function verifyRootContents(path,allowedDirs,allowedFiles) {
  const entries=opendirSync(path);
  try {
    let entry;
    while((entry=entries.readSync())!==null){
      const child=lstatSync(join(path,entry.name));
      if(child.isSymbolicLink())throw Error('Retired Cargo archive contains a link');
      if(child.isDirectory()? !allowedDirs.has(entry.name) : !child.isFile() || !allowedFiles.has(entry.name) || child.nlink!==1){
        throw Error('Retired Cargo archive contains an unrecognized entry');
      }
    }
  }finally{entries.closeSync();}
}

export function inspectArchivedCargo(root,options={}) {
  const checkout=resolve(root);
  const parent=admitChain(checkout,'workspace/archive/compiler-cache-20261026');
  if(!parent)return {status:'ABSENT',logicalBytes:'0',reason:'Compiler archive is not present'};
  // Never delete even a known target if the archive contains other material.
  verifyRootContents(parent,new Set(['target']),new Set());
  const archived=admitChain(checkout,ARCHIVE);
  if(!archived)return {status:'ABSENT',logicalBytes:'0',reason:'Historical Cargo target is not present'};
  verifyRootContents(archived,ROOT_DIRECTORY_NAMES,ROOT_FILE_NAMES);
  const live=admitChain(checkout,CARGO_ROOTS[0].path);
  if(!live)return {status:'BLOCKED',logicalBytes:'0',reason:'Canonical active Cargo target is absent; rebuild verification required'};
  const scan=options.inspect??inspectCargoFootprint;
  const knobs={maxEntries:options.maxEntries??MAX_ENTRIES,maxDurationMs:options.maxDurationMs??MAX_SCAN_MS};
  const active=scan(checkout,{roots:[CARGO_ROOTS[0]],...knobs});
  const retired=scan(checkout,{roots:[APPROVED_ROOTS[0]],...knobs});
  const liveRow=active.rows?.[0],oldRow=retired.rows?.[0];
  const windowsDeps=(liveRow?.buckets?.get('Windows MSVC debug / deps')?.files??0)+
    (liveRow?.buckets?.get('Windows MSVC release / deps')?.files??0);
  if(!active.complete || liveRow?.status!=='OK' || liveRow.bytes<1n || windowsDeps<1){
    return {status:'BLOCKED',logicalBytes:'0',
      reason:'Canonical Windows MSVC compiler output is not complete; a replacement build must exist'};
  }
  if(!retired.complete || oldRow?.status!=='OK' || oldRow.hardlinkedFiles!==0 ||
     oldRow.links!==0 || oldRow.errors!==0 || oldRow.files<1){
    return {status:'BLOCKED',logicalBytes:String(oldRow?.bytes??0n),
      reason:'Historical Cargo target is incomplete, linked, empty or untrusted'};
  }
  return {status:'REVIEWABLE',logicalBytes:String(oldRow.bytes),
    reason:'Only historical Cargo artifact target; no source, private data, or active cache'};
}

export function retireArchivedCargo(root,options={}) {
  let admission;
  try{admission=inspectArchivedCargo(root,options);}
  catch{return {status:'BLOCKED',reason:'Compiler archive path/contents could not be admitted'};}
  if(admission.status!=='REVIEWABLE')return admission;
  const checkout=resolve(root);
  const target=join(checkout,...ARCHIVE.split('/'));
  const cargo=join(checkout,'workspace/toolchains/cargo/bin/cargo.exe');
  const manifest=join(checkout,'Development/apps/desktop/src-tauri/Cargo.toml');
  const toolchainPath=join(checkout,'Development/toolchain.json');
  try {
    const info=lstatSync(cargo),manifestInfo=lstatSync(manifest),toolInfo=lstatSync(toolchainPath);
    if(!info.isFile() || info.isSymbolicLink() || !manifestInfo.isFile() ||
       manifestInfo.isSymbolicLink() || !toolInfo.isFile() ||
       toolInfo.isSymbolicLink() || toolInfo.size>32768)throw Error('Untrusted Cargo runner/manifest');
    const tc=JSON.parse(readFileSync(toolchainPath,'utf8'));
    if(!/^\d+\.\d+\.\d+$/.test(tc?.rust?.version??''))throw Error('Pinned Cargo version is missing');
    // Ensure archive root is still the same fixed, ordinary directory.
    if(!admitChain(checkout,ARCHIVE))throw Error('Archive target disappeared');
    verifyRootContents(join(checkout,'workspace/archive/compiler-cache-20261026'),new Set(['target']),new Set());
    const args=['clean','--locked','--manifest-path',manifest,'--target-dir',target];
    const env={...process.env,CARGO_TARGET_DIR:target,CARGO_BUILD_TARGET_DIR:target,CARGO_BUILD_BUILD_DIR:target};
    const invoke=options.execute??((bin,argv,opts)=>spawnSync(bin,argv,opts));
    const version=invoke(cargo,['--version'],{cwd:join(checkout,'Development'),env,
      encoding:'utf8',shell:false,windowsHide:true,timeout:15000});
    if(version.error || version.status!==0 ||
       !version.stdout?.startsWith('cargo '+tc.rust.version+' ')) {
      return {status:'BLOCKED',reason:'Pinned Cargo executable version was not verified'};
    }
    const result=invoke(cargo,args,{cwd:join(checkout,'Development'),env,shell:false,
      windowsHide:true,stdio:options.silent?'pipe':'inherit',timeout:600000});
    if(result.error || result.status!==0)return {status:'FAILED',reason:'Scoped cargo clean was not successful'};
    const after=admitChain(checkout,ARCHIVE);
    if(after!==null) {
      const check=(options.inspect??inspectCargoFootprint)(checkout,{
        roots:[APPROVED_ROOTS[0]],maxEntries:MAX_ENTRIES,maxDurationMs:MAX_SCAN_MS});
      if(!check.complete || check.rows[0].status!=='OK' || check.rows[0].bytes!==0n){
        return {status:'PARTIAL',reason:'Archived target still contains compiler output; cleanup not proven complete'};
      }
    }
    return {status:'RETIRED',logicalBytesBefore:admission.logicalBytes,
      reason:'Scoped Cargo cleanup completed; logical bytes are not observed NTFS disk savings'};
  }catch{
    return {status:'BLOCKED',reason:'Unable to verify scoped Cargo cleanup preconditions'};
  }
}

// Only the reviewed AI task registry can dispatch this without arbitrary args.
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  if(process.argv.length!==2){console.error('No direct arguments accepted; use the approved DEV.cmd task');process.exitCode=2;}
  else{
    const root=resolve(fileURLToPath(new URL('../../../',import.meta.url)));
    const result=retireArchivedCargo(root);
    console.log(JSON.stringify(result));
    process.exitCode=['RETIRED','ABSENT'].includes(result.status)?0:3;
  }
}
