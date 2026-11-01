// Explicit offline .mcpack packager for the ONE reusable Education pack.
// No Minecraft launch, profile/world search, network/AI access or input.
// Refuses replacing any existing archive; source-controlled pack UUIDs stay
// identical across different test worlds. UTF-8 fixed file names only.
import {createHash} from 'node:crypto';
import {lstat,mkdir,open,readFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const PACK_NAME='LazyTester-Universal-Diagnostics-Education-1.0.0.mcpack';
const SOURCE=new URL('./education-universal-pack/',import.meta.url);
const ROOT=fileURLToPath(new URL('../../../',import.meta.url));
const ENTRIES=['manifest.json','scripts/main.js'];
const MAX_SOURCE_BYTES=64*1024;
const MAX_ARCHIVE_BYTES=256*1024;

function digest(bytes){return createHash('sha256').update(bytes).digest('hex');}
function crc32(buf){
  let crc=0xffffffff;
  for(let i=0;i<buf.length;i++){
    crc^=buf[i];
    for(let j=0;j<8;j++)crc=(crc>>>1)^((crc&1)?0xedb88320:0);
  }
  return (crc^0xffffffff)>>>0;
}
// ZIP STORE is sufficient for this tiny file. No external archiver, JS
// dependency or dynamic paths. Fixed DOS timestamp makes builds stable.
export function buildEducationMcpack(entries){
  if(!Array.isArray(entries)||entries.length!==2||
     entries[0]?.name!=='manifest.json'||
     entries[1]?.name!=='scripts/main.js') {
    throw new TypeError('Unexpected Education pack entries');
  }
  const locals=[],central=[];
  let offset=0;
  for(const item of entries){
    if(!Buffer.isBuffer(item.content)||item.content.length<1||
       item.content.length>MAX_SOURCE_BYTES)throw new TypeError('Invalid pack content');
    const name=Buffer.from(item.name,'ascii');
    const body=item.content,crc=crc32(body);
    const local=Buffer.alloc(30);
    local.writeUInt32LE(0x04034b50,0);
    local.writeUInt16LE(20,4);
    local.writeUInt16LE(0,6); // UTF-8 not needed: both names ASCII
    local.writeUInt16LE(0,8); // ZIP store, not deflate
    local.writeUInt16LE(0,10); // DOS time
    local.writeUInt16LE(33,12); // DOS 1980-01-01
    local.writeUInt32LE(crc,14);
    local.writeUInt32LE(body.length,18);
    local.writeUInt32LE(body.length,22);
    local.writeUInt16LE(name.length,26);
    local.writeUInt16LE(0,28);
    locals.push(local,name,body);
    const itemCentral=Buffer.alloc(46);
    itemCentral.writeUInt32LE(0x02014b50,0);
    itemCentral.writeUInt16LE(20,4); // version made by
    itemCentral.writeUInt16LE(20,6); // version needed
    itemCentral.writeUInt16LE(0,8);
    itemCentral.writeUInt16LE(0,10);
    itemCentral.writeUInt16LE(0,12);
    itemCentral.writeUInt16LE(33,14);
    itemCentral.writeUInt32LE(crc,16);
    itemCentral.writeUInt32LE(body.length,20);
    itemCentral.writeUInt32LE(body.length,24);
    itemCentral.writeUInt16LE(name.length,28);
    itemCentral.writeUInt16LE(0,30);
    itemCentral.writeUInt16LE(0,32);
    itemCentral.writeUInt16LE(0,34);
    itemCentral.writeUInt16LE(0,36);
    itemCentral.writeUInt32LE(0,38);
    itemCentral.writeUInt32LE(offset,42);
    central.push(itemCentral,name);
    offset+=local.length+name.length+body.length;
  }
  const dir=Buffer.concat(central);
  const end=Buffer.alloc(22);
  end.writeUInt32LE(0x06054b50,0);
  end.writeUInt16LE(entries.length,8);
  end.writeUInt16LE(entries.length,10);
  end.writeUInt32LE(dir.length,12);
  end.writeUInt32LE(offset,16);
  const packed=Buffer.concat([...locals,dir,end]);
  if(packed.length>MAX_ARCHIVE_BYTES)throw Error('Education .mcpack exceeds budget');
  return packed;
}
async function ordinary(dir){
  const s=await lstat(dir);
  if(!s.isDirectory()||s.isSymbolicLink())throw Error('Untrusted staging directory');
}
async function child(parent,name){
  await ordinary(parent);
  const p=path.join(parent,name);
  try{await mkdir(p,{mode:0o700});}
  catch(e){if(e.code!=='EEXIST')throw e;}
  await ordinary(p);
  return p;
}
async function pinnedSource(rel){
  const base=fileURLToPath(SOURCE);
  const file=path.join(base,rel);
  const s=await lstat(file);
  if(!s.isFile()||s.isSymbolicLink()||s.nlink!==1||
     s.size<1||s.size>MAX_SOURCE_BYTES)throw Error('Untrusted pack source');
  const content=await readFile(file);
  const after=await lstat(file);
  if(s.dev!==after.dev||s.ino!==after.ino||s.size!==after.size||
     s.mtimeMs!==after.mtimeMs)throw Error('Pack source changed');
  return content;
}
export async function stageUniversalEducationMcpack(){
  const contents=await Promise.all(ENTRIES.map(pinnedSource));
  const manifest=JSON.parse(contents[0].toString('utf8'));
  if(manifest.format_version!==2||
     manifest.header?.uuid!=='4af9d3f0-9fa1-4cb9-a011-fd37c1b2fb16'||
     manifest.header?.version?.join('.')!=='1.0.0'||
     manifest.modules?.[1]?.entry!=='scripts/main.js'||
     manifest.dependencies?.[0]?.module_name!=='@minecraft/server'||
     manifest.dependencies?.[0]?.version!=='2.8.0') {
    throw Error('Reviewed reusable Education manifest drift');
  }
  const content=buildEducationMcpack(ENTRIES.map((name,i)=>({
    name,content:contents[i]
  })));
  const dir=await child(await child(await child(
    path.join(ROOT,'workspace'),'diagnostics'),'education-universal'),'v1');
  const file=path.join(dir,PACK_NAME);
  const hash=digest(content);
  try{
    const h=await open(file,'wx',0o600);
    try{await h.writeFile(content);await h.sync();}
    finally{await h.close();}
    return {status:'CREATED_LOCALLY_NOT_EDUCATION_TESTED',
      relativeFile:'workspace/diagnostics/education-universal/v1/'+PACK_NAME,
      bytes:content.length,sha256:hash,uploadedToAI:false};
  }catch(error){
    if(error?.code!=='EEXIST')throw error;
    const info=await lstat(file);
    if(!info.isFile()||info.isSymbolicLink()||info.nlink!==1||
       info.size!==content.length||info.size>MAX_ARCHIVE_BYTES) {
      throw Error('Existing pack differs; never overwrite');
    }
    const previous=await readFile(file);
    if(!previous.equals(content))throw Error('Existing pack differs; never overwrite');
    return {status:'ALREADY_PRESENT_IDENTICAL_NOT_EDUCATION_TESTED',
      relativeFile:'workspace/diagnostics/education-universal/v1/'+PACK_NAME,
      bytes:content.length,sha256:hash,uploadedToAI:false};
  }
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  if(process.argv.length!==3||process.argv[2]!=='prepare'){
    console.error('Usage: node build-education-universal-pack.mjs prepare');
    process.exitCode=2;
  }else{
    stageUniversalEducationMcpack().then(result=>console.log(JSON.stringify(result)))
      .catch(()=>{console.error('Education universal pack staging refused');process.exitCode=1;});
  }
}
