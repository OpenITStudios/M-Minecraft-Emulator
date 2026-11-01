// Source-only Minecraft Education Script API pack STAGER.
// Explicit CLI; never installs/enables a pack, locates game profiles/worlds,
// opens a debugger, collects logs, calls AI, or touches private account data.
// Runtime support for any declared stable API must be checked in Education.
import {createHash,randomBytes,randomUUID} from 'node:crypto';
import {lstat,mkdir,open,readFile,readdir} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const UUID=/^[a-f0-9]{8}-[a-f0-9]{4}-4[a-f0-9]{3}-[89ab][a-f0-9]{3}-[a-f0-9]{12}$/;
const RUN=/^edu-[a-f0-9]{12}$/;
// The stable API dependency must be chosen explicitly after reviewing the
// targeted Education build. This only checks semver syntax, NOT availability.
const STABLE=/^(?:1|2)\.(?:0|[1-9][0-9]?)\.(?:0|[1-9][0-9]?)$/;
const MAX_SOURCE_BYTES=4096;
const MAX_PACKS=64;
const ORIGINAL_PROBE=new URL('./education-script-probe.js',import.meta.url);
const REPO_ROOT=fileURLToPath(new URL('../../../',import.meta.url));

function sha256(text) {
  return createHash('sha256').update(text,'utf8').digest('hex');
}
function checkUuid(value) {
  if(typeof value!=='string'||!UUID.test(value)) {
    throw new TypeError('Invalid UUID for Education probe pack');
  }
}
export function buildEducationProbePack({run,apiVersion,probeSource,uuids}={}) {
  if(typeof run!=='string'||!RUN.test(run)) {
    throw new TypeError('Expected generated edu-<12 lowercase hex> run');
  }
  if(typeof apiVersion!=='string'||!STABLE.test(apiVersion)) {
    throw new TypeError('Choose an explicit stable @minecraft/server version');
  }
  if(typeof probeSource!=='string'||
     Buffer.byteLength(probeSource,'utf8')>MAX_SOURCE_BYTES ||
     !probeSource.includes("import {system} from '@minecraft/server';") ||
     !probeSource.includes('export function emitEducationScriptLogProbe(run)') ||
     !probeSource.includes("emit(1,'START')") ||
     !probeSource.includes("emit(2,'DEFERRED')") ||
     !probeSource.endsWith('\n')) {
    throw new TypeError('Source probe does not match the reviewed emitter shape');
  }
  if(!Array.isArray(uuids)||uuids.length!==3||
     new Set(uuids).size!==3) {
    throw new TypeError('Three distinct UUIDs required');
  }
  uuids.forEach(checkUuid);

  // Manifest v2 is deliberately used: v3 is not required for the basic
  // stable-script probe. Version declared here is NOT runtime compatibility.
  const manifest={
    format_version:2,
    header:{
      name:'Lazy Tester Education Probe '+run,
      description:'Isolated QA world only. Two Script API log markers. No gameplay changes.',
      uuid:uuids[0],version:[1,0,0],
      min_engine_version:apiVersion.startsWith('2.')?
        [1,21,90]:[1,21,60],
    },
    modules:[
      {type:'data',uuid:uuids[1],version:[1,0,0]},
      {type:'script',language:'javascript',uuid:uuids[2],
        version:[1,0,0],entry:'scripts/main.js'},
    ],
    dependencies:[{module_name:'@minecraft/server',version:apiVersion}],
  };
  const manifestText=JSON.stringify(manifest,null,2)+'\n';
  const mainJs=probeSource+
    '\n// Activate only in a deliberate isolated test-copy world.\n'+
    'emitEducationScriptLogProbe('+JSON.stringify(run)+');\n';
  return {run,apiVersion,manifest,manifestText,mainJs};
}
async function ordinaryDirectory(dir) {
  const stat=await lstat(dir);
  if(!stat.isDirectory()||stat.isSymbolicLink()) {
    throw new Error('Staging path is not an ordinary directory');
  }
}
async function ensureChild(parent,name) {
  await ordinaryDirectory(parent);
  const child=path.join(parent,name);
  try{await mkdir(child,{mode:0o700});}
  catch(e){if(e.code!=='EEXIST')throw e;}
  await ordinaryDirectory(child);
  return child;
}
async function writeExclusive(file,content) {
  const handle=await open(file,'wx',0o600);
  try{await handle.writeFile(content,'utf8');await handle.sync();}
  finally{await handle.close();}
}
// No custom output root/path; workspace/ is the repo's ignored output
// location. A generated run ID prevents collisions with previous attempts.
export async function stageEducationProbe({apiVersion}={}) {
  if(typeof apiVersion!=='string'||!STABLE.test(apiVersion)) {
    throw new TypeError('An explicit stable @minecraft/server version is required');
  }
  const source=await readFile(ORIGINAL_PROBE,'utf8');
  if(Buffer.byteLength(source,'utf8')>MAX_SOURCE_BYTES) {
    throw new Error('Reviewed probe source exceeded the byte budget');
  }
  const run='edu-'+randomBytes(6).toString('hex');
  const built=buildEducationProbePack({
    run,apiVersion,probeSource:source,
    uuids:[randomUUID(),randomUUID(),randomUUID()],
  });
  const workspace=path.join(REPO_ROOT,'workspace');
  const diagnostics=await ensureChild(workspace,'diagnostics');
  const probes=await ensureChild(diagnostics,'education-probe');
  const previous=await readdir(probes);
  if(previous.length>=MAX_PACKS)throw new Error('Offline probe staging quota reached');
  const dest=path.join(probes,run);
  // Exclusive creation: no automatic delete, overwrite or partial-run reuse.
  await mkdir(dest,{mode:0o700});
  const pack=await ensureChild(dest,'pack');
  const scripts=await ensureChild(pack,'scripts');
  await writeExclusive(path.join(scripts,'main.js'),built.mainJs);
  await writeExclusive(path.join(pack,'manifest.json'),built.manifestText);
  const receipt={
    schema:1,run,apiVersion,source:'REVIEWED_SCRIPT_API_SOURCE_ONLY',
    manifestSha256:sha256(built.manifestText),mainSha256:sha256(built.mainJs),
    status:'STAGED_NOT_INSTALLED_NOT_EXECUTED',
    minecraftEducationApi:'NOT_VERIFIED',world:'NOT_SELECTED',
    logTransport:'NOT_VERIFIED',rawDataSharedWithAI:false,
  };
  // Final source-side commit marker outside the importable pack folder.
  await writeExclusive(path.join(dest,'receipt.json'),
    JSON.stringify(receipt,null,2)+'\n');
  return {...receipt,
    relativePackFolder:'workspace/diagnostics/education-probe/'+run+'/pack'};
}
if(process.argv[1]&&path.resolve(process.argv[1])===
   fileURLToPath(import.meta.url)) {
  const args=process.argv.slice(2);
  if(args.length!==3||args[0]!=='prepare'||args[1]!=='--server-version') {
    console.error('Usage: node prepare-education-probe.mjs prepare --server-version <stable version>');
    process.exitCode=2;
  }else{
    stageEducationProbe({apiVersion:args[2]}).then(result=>{
      console.log(JSON.stringify(result));
    }).catch(()=>{
      // Never print unexpected system paths or source/pack contents.
      console.error('Education test-pack staging refused; inspect local workspace manually');
      process.exitCode=1;
    });
  }
}
