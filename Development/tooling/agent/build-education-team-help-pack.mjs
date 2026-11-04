// Explicit offline .mcpack packager for a HELP-ONLY command list.
// No Minecraft launch, profile/world search, network/AI access or input.
// Refuses replacing any existing archive; source-controlled pack UUIDs stay
// identical across different test worlds; no analytics or log engine. UTF-8 fixed file names only.
import {createHash} from 'node:crypto';
import {lstat,mkdir,open,readFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const PACK_NAME='LazyTester-Team-Commands-Help-Education-2.2.0.mcpack';
const SOURCE=new URL('./education-team-help-pack/',import.meta.url);
const ROOT=fileURLToPath(new URL('../../../',import.meta.url));
const ENTRIES=[
  'manifest.json',
  'functions/help.mcfunction',
  'functions/lazytester/help.mcfunction',
];
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
  if(!Array.isArray(entries)||entries.length!==ENTRIES.length||
     entries.some((entry,i)=>entry?.name!==ENTRIES[i])) {
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
     manifest.header?.version?.join('.')!=='2.2.0'||
     manifest.header?.name!=='Lazy Tester - Team Commands Help'||
     !Array.isArray(manifest.modules)||manifest.modules.length!==1||
     manifest.modules[0]?.type!=='data'||
     manifest.modules[0]?.uuid!=='736d52ad-7c03-4a4b-880d-25a89072586f'||
     manifest.modules[0]?.version?.join('.')!=='2.2.0'||
     Object.hasOwn(manifest,'dependencies')) {
    throw Error('Reviewed reusable Education manifest drift');
  }
  // No JavaScript module, dependencies, dynamic script-event listener, or
  // analytics: the one static function displays developer command help.
  const alias=contents[1].toString('utf8');
  const aliasCommands=alias.split(/\r?\n/).filter(line=>
    line.trim()&&!line.startsWith('#'));
  if(aliasCommands.length!==1||aliasCommands[0]!=='function lazytester/help') {
    throw Error('Short help alias must forward only to the canonical function');
  }
  const lines=contents[2].toString('utf8').split(/\r?\n/)
    .filter(line=>line.trim()&&!line.startsWith('#'));
  if(lines.length<5||lines.length>18) {
    throw Error('Help display has unsupported number of commands');
  }
  const messages=[];
  for(const line of lines) {
    if(!line.startsWith('tellraw @s ')) {
      throw Error('Help pack cannot execute non-display commands');
    }
    let obj;
    try {obj=JSON.parse(line.slice('tellraw @s '.length));}
    catch {throw Error('Invalid help message JSON');}
    if(!obj||Object.keys(obj).length!==1||
       !Array.isArray(obj.rawtext)||obj.rawtext.length!==1||
       Object.keys(obj.rawtext[0]).length!==1||
       typeof obj.rawtext[0].text!=='string'||
       obj.rawtext[0].text.length>220) {
      throw Error('Help display must be a fixed, bounded single text line');
    }
    messages.push(obj.rawtext[0].text);
  }
  const rendered=messages.join('\n');
  for(const cmd of [
    '/scriptevent mivubi:get_analytics',
    '/scriptevent daigon:get_all_scoring',
    '/scriptevent probe:list',
    '/scriptevent probe:run auto',
    '/scriptevent probe:report',
  ]) {
    if(!rendered.includes(cmd))throw Error('Canonical team command is missing');
  }
  const content=buildEducationMcpack(ENTRIES.map((name,i)=>({
    name,content:contents[i]
  })));
  const dir=await child(await child(await child(
    path.join(ROOT,'workspace'),'diagnostics'),'education-team-help'),'v2.2');
  const file=path.join(dir,PACK_NAME);
  const hash=digest(content);
  try{
    const h=await open(file,'wx',0o600);
    try{await h.writeFile(content);await h.sync();}
    finally{await h.close();}
    return {status:'CREATED_LOCALLY_NOT_EDUCATION_TESTED',
      relativeFile:'workspace/diagnostics/education-team-help/v2.2/'+PACK_NAME,
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
      relativeFile:'workspace/diagnostics/education-team-help/v2.2/'+PACK_NAME,
      bytes:content.length,sha256:hash,uploadedToAI:false};
  }
}
if(process.argv[1]&&path.resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  if(process.argv.length!==3||process.argv[2]!=='prepare'){
    console.error('Usage: node build-education-team-help-pack.mjs prepare');
    process.exitCode=2;
  }else{
    stageUniversalEducationMcpack().then(result=>console.log(JSON.stringify(result)))
      .catch(()=>{console.error('Education team-help pack staging refused');process.exitCode=1;});
  }
}
