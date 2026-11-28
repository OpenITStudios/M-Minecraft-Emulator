// Offline, read-only projection of the existing bounded desktop report.
import {lstatSync, fstatSync, openSync, readSync, closeSync} from 'node:fs';
import {join, dirname} from 'node:path';

const MAX_BYTES=16*1024, MAX_ROWS=12;
const OWNERS=Object.freeze({
  "ui": "Development/apps/desktop/src/LazyTesterApp.svelte",
  "bootstrap": "Development/apps/desktop/src-tauri/src/app_bootstrap.rs",
  "binding": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/profile_binding.rs",
  "transport": "Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs",
  "session": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs",
  "workspace": "Development/apps/desktop/src-tauri/src/engine/workspace_surface/controller.rs",
  "preview": "Development/apps/desktop/src-tauri/src/engine/workspace_surface/previews.rs",
  "profiles": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/product_profiles.rs",
  "storage": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/profile_storage.rs",
  "clear": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/product_profiles/clear.rs",
  "settings": "Development/apps/desktop/src-tauri/src/engine/manager_settings.rs",
  "audio": "Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_audio.rs",
  "screenshot": "Development/apps/desktop/src-tauri/src/engine/capture/screenshot.rs",
  "recording": "Development/apps/desktop/src-tauri/src/engine/capture/recording.rs",
  "agent": "Development/tooling/agent/operation-core.mjs",
  "probe": "Development/tooling/agent/probe-qa.mjs",
  "activity": "Development/tooling/agent/activity-status.mjs"
});
const ROUTES=[
  ["LT-APP-01 LT-APP-03",2,"ui","Check mounted action/observation admission and event delivery; do not infer native failure."],
  ["LT-APP-02",2,"bootstrap","Check native capability receipt and startup gates."],
  ["LT-APP-04",1,"bootstrap","Check close handshake and the original exit receipt before another close."],
  ["LT-APP-05 LT-SET-05 LT-SET-06",3,"ui","Check expected refusal; do not remove the admission gate."],
  ["LT-PRO-01 LT-PRO-02",2,"binding","Check profile/catalog admission and actual observed failure; preserve data."],
  ["LT-CON-01",2,"transport","Check whether current observation failed in transport, receipt parsing, or workspace release."],
  ["LT-GAM-01 LT-GAM-05",2,"session","Check original process/window observation and launch admission."],
  ["LT-GAM-02 LT-GAM-03 LT-GAM-04",1,"session","Check normal-close and retained ownership receipts; never replay an uncertain close."],
  ["LT-DIS-01 LT-DIS-02 LT-DIS-05 LT-DIS-06",2,"workspace","Check selected window identity and presentation receipt."],
  ["LT-DIS-03",1,"workspace","Check original restoration checkpoint and helper ownership; Inspect alone is not restore proof."],
  ["LT-DIS-04",3,"preview","Check exact-target DWM registration and invalidation."],
  ["LT-STO-01 LT-STO-07",2,"profiles","Check verified data-location or folder receipt without accessing private data."],
  ["LT-STO-02 LT-STO-03 LT-STO-04 LT-STO-05",2,"storage","Check selected storage scope, admission, and receipt; never retry uncertain deletion."],
  ["LT-STO-06",1,"clear","Check pending cleanup journal and verified disposition; preserve account/world data."],
  ["LT-SET-01 LT-SET-04",2,"session","Check settings operation and selected profile receipt."],
  ["LT-SET-02 LT-SET-07",2,"settings","Check validated preferences and actual save receipt."],
  ["LT-SET-03",2,"audio","Check audio synchronization and bounded fallback; do not infer COM root cause."],
  ["LT-CAP-01",3,"screenshot","Check selected capture receipt and no-clobber save; cancellation is not failure."],
  ["LT-CAP-02 LT-CAP-03 LT-CAP-04 LT-CAP-05",2,"recording","Check recording admission, worker result and finalization/recovery receipt."],
  ["LT-AGT-01 LT-AGT-02",1,"agent","Check original lease/neutral and reconciliation evidence; never dispatch a replacement input."],
  ["LT-AGT-03 LT-AGT-04",2,"probe","Check original Probe receipt completeness and checkpoint origin, not gameplay PASS."],
  ["LT-AGT-05",3,"activity","Check report freshness and origin; stale evidence is not current failure proof."],
  ["IPC_WORKER_START_FAILED",2,"transport","Check fallible worker admission and numeric OS error; no worker was admitted."],
  ["IPC_TRANSPORT_FAILED",1,"transport","Check exact transport failure and uncertainty gate; do not infer whether mutation reached actor."],
  ["IPC_INPUT_RECEIPT_INVALID IPC_TARGET_RECEIPT_INVALID IPC_INSPECT_RECEIPT_INVALID",1,"transport","Check exact receipt validator and original request/target contract; keep uncertainty gate."],
  ["IPC_REPLY_TIMEOUT IPC_REPLY_LOST",1,"transport","Check original worker and bounded reply handling; full Inspect reconciles transport, not original operation."],
];
const CATALOG=new Map(ROUTES.flatMap(([codes,priority,owner,check])=>
  codes.split(' ').map(code=>[code,{priority,owner,check}])));
const NATIVE_OPS=new Set(["inspect","target","agent_activity","testing_reserve","testing_finalize","testing_inspect","inspect_settings","apply_preset","apply_graphics_preset","save_performance","save_audio","save_storage_preferences","inspect_storage","inspect_profile_storage","inspect_selective_inventory","review_selective_inventory","confirm_selected_worlds","confirm_selected_packs","open_profile_folder","clear_profile_cache","clear_profile_download_cache","reset_game_settings","restore_game_settings","disable_improved_input_response","enable_texture_streaming","disable_texture_streaming","clear_profile_data","launch","launch_mc02_experiment","launch_ai_experiment","mc02_input","ai_leased_input","ai_lease_acquire","ai_lease_release","ai_input_group","close_one","close_all"]);
const record=v=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const own=(v,k)=>Object.hasOwn(v,k);
const uint=(v,max=Number.MAX_SAFE_INTEGER)=>Number.isSafeInteger(v)&&v>=0&&v<=max;
const positive=v=>uint(v,0xffffffff)&&v>0;
const exact=(v,required,optional=[])=>record(v)&&required.every(k=>own(v,k))&&
  Object.keys(v).every(k=>required.includes(k)||optional.includes(k));
function invalid(){return Error('Invalid or unsupported issue report');}
function flag(v,k){return !own(v,k)||v[k]===true;}
export function issueCatalogCodes(){return [...CATALOG.keys()];}

function parseRows(rows,native){
  if(!Array.isArray(rows)||rows.length>(native?16:48))throw invalid();
  const seen=new Set();
  for(const row of rows){
    const required=native?['c','op','t','n']:['c','t'];
    const optional=native?['s','os','l','r','e']:['s','n','l','r','e'];
    if(!exact(row,required,optional)||typeof row.c!=='string'||!CATALOG.has(row.c)||
      row.c.startsWith('IPC_')!==native||!uint(row.t)||
      (own(row,'s')&&(!uint(row.s,5)||row.s===0))||
      (own(row,'n')&&!positive(row.n))||
      (own(row,'e')&&!positive(row.e))||
      (own(row,'l')&&!uint(row.l))||(own(row,'r')&&!uint(row.r)))throw invalid();
    if(native){
      if(!NATIVE_OPS.has(row.op)||
        (own(row,'os')&&(!Number.isInteger(row.os)||row.os< -2147483648||row.os>2147483647))||
        (row.n>1)!==own(row,'l')||
        (own(row,'e')&&row.e>=row.n&&row.n!==0xffffffff)||
        (row.c==='IPC_WORKER_START_FAILED'&&(own(row,'r')||own(row,'e'))))throw invalid();
    }
    const key=JSON.stringify(native?[row.c,row.op,row.s??null,row.os??null]:[row.c,row.s??null]);
    if(seen.has(key))throw invalid();
    seen.add(key);
  }
  return rows;
}

export function reviewIssueReport(text){
  if(typeof text!=='string'||Buffer.byteLength(text)>MAX_BYTES)throw invalid();
  let report;
  try{report=JSON.parse(text);}catch{throw invalid();}
  if(!exact(report,['v','scope','issues'],['native','nativeUnavailable','truncated','storageUnavailable'])||
    report.v!==2||report.scope!=='desktop-process'||
    !['nativeUnavailable','truncated','storageUnavailable'].every(k=>flag(report,k))||
    own(report,'native')===own(report,'nativeUnavailable'))throw invalid();
  const ui=parseRows(report.issues,false);
  let native=[];
  if(own(report,'native')&&report.native!==null){
    const n=report.native;
    if(!exact(n,['v','issues'],['truncated'])||n.v!==1||!flag(n,'truncated'))throw invalid();
    native=parseRows(n.issues,true);
    if(!native.length)throw invalid();
  }
  if(!ui.length&&!native.length)throw invalid();
  const candidates=[...native.map(row=>({row,producer:'native-request-client'})),
    ...ui.map(row=>({row,producer:'desktop-ui'}))].map(({row,producer})=>{
      const route=CATALOG.get(row.c);
      return {row,producer,...route};
    });
  // Rank review urgency, not proven severity or independent incident count.
  candidates.sort((a,b)=>a.priority-b.priority||
    Number(own(a.row,'r'))-Number(own(b.row,'r'))||
    (b.row.e??0)-(a.row.e??0)||(b.row.n??0)-(a.row.n??0)||
    a.row.c.localeCompare(b.row.c)||(a.row.s??0)-(b.row.s??0)||
    (a.row.op??'').localeCompare(b.row.op??'')||(a.row.os??0)-(b.row.os??0));
  const selected=candidates.slice(0,MAX_ROWS);
  // One source path and instruction per route, not repeated for each row.
  const routes=[];
  const routeIds=new Map();
  const issues=selected.map(({row,producer,priority,owner,check})=>{
    const key=JSON.stringify([priority,owner,check]);
    if(!routeIds.has(key)){
      routeIds.set(key,routes.length);
      routes.push({priority,source:OWNERS[owner],check});
    }
    return {route:routeIds.get(key),producer,...row,
      recovery:own(row,'r')?(producer==='native-request-client'?'TRANSPORT_RECONCILED':'POSITIVE_RECEIPT_OBSERVED'):'UNCONFIRMED'};
  });
  return {status:'REVIEW_CANDIDATES',proof:'LOCAL_REPORT_NOT_RUNTIME_ATTESTATION',
    reportedBuild:'UNKNOWN',rootCause:'NOT_ESTABLISHED',
    coverage:{uiRows:ui.length,nativeRows:own(report,'nativeUnavailable')?null:native.length,
      truncated:report.truncated===true||report.native?.truncated===true,
      ...(report.storageUnavailable?{storageUnavailable:true}:{}),
      omitted:candidates.length-selected.length},
    routes,issues};
}

function sameFile(a,b){
  return a.dev===b.dev&&a.ino===b.ino&&a.size===b.size&&
    a.mtimeNs===b.mtimeNs&&a.ctimeNs===b.ctimeNs;
}
function ordinaryDirectory(path){
  const stat=lstatSync(path);
  if(!stat.isDirectory()||stat.isSymbolicLink())throw invalid();
}
// The caller supplies the trusted checkout root; CLI has no arbitrary path flag.
// No enumeration, staging read, process discovery, native IPC or write.
export function readIssueReport(root){
  const file=join(root,'Data','Reports','issues.json');
  const parent=dirname(file);
  try{
    ordinaryDirectory(root);
    ordinaryDirectory(join(root,'Data'));
    ordinaryDirectory(parent);
  }catch(error){if(error?.code==='ENOENT')return null;throw invalid();}
  let before;
  try{before=lstatSync(file,{bigint:true});}
  catch(error){if(error?.code==='ENOENT')return null;throw invalid();}
  if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1n||
    before.size>BigInt(MAX_BYTES))throw invalid();
  let fd;
  try{
    fd=openSync(file,'r');
    const opened=fstatSync(fd,{bigint:true});
    if(!opened.isFile()||opened.nlink!==1n||!sameFile(before,opened))throw invalid();
    const bytes=Buffer.alloc(MAX_BYTES+1);
    let total=0;
    while(total<bytes.length){
      const count=readSync(fd,bytes,total,bytes.length-total,null);
      if(count===0)break;
      total+=count;
    }
    if(total>MAX_BYTES||BigInt(total)!==opened.size||
      !sameFile(opened,fstatSync(fd,{bigint:true})))throw invalid();
    const named=lstatSync(file,{bigint:true});
    if(!named.isFile()||named.isSymbolicLink()||named.nlink!==1n||!sameFile(opened,named))throw invalid();
    ordinaryDirectory(join(root,'Data'));
    ordinaryDirectory(parent);
    return new TextDecoder('utf-8',{fatal:true}).decode(bytes.subarray(0,total));
  }catch{throw invalid();}
  finally{if(fd!==undefined)closeSync(fd);}
}
