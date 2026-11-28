// Explicit, offline artifact metadata only. Never a game launch or admission route.
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createReadStream, lstatSync, realpathSync, openSync, closeSync, readSync, fstatSync, constants} from 'node:fs';
import {isAbsolute, join, resolve, basename} from 'node:path';
import {pathToFileURL} from 'node:url';

const dictionary = value => value !== null && typeof value === 'object' && !Array.isArray(value);
// Observed in the user-supplied 1.26.33.0 bundle metadata; not a signer pin.
const expectedBundleId = 'com.microsoft.minecraft-edu';
const expectedExecutable = 'minecraft-edu';
const version = /^\d+(?:\.\d+){0,4}$/;
const validVersion = value => typeof value==='string' && value.length<=64 && version.test(value);

export function bundleMetadata(plist) {
  if (!dictionary(plist) || plist.CFBundlePackageType !== 'APPL' ||
      plist.CFBundleIdentifier !== expectedBundleId ||
      plist.CFBundleExecutable !== expectedExecutable ||
      !validVersion(plist.CFBundleVersion) || !validVersion(plist.CFBundleShortVersionString))
    throw Error('INVALID_BUNDLE_METADATA');
  for (const key of ['LSMultipleInstancesProhibited','LSRequiresNativeExecution'])
    if (Object.hasOwn(plist,key) && typeof plist[key] !== 'boolean') throw Error('INVALID_BUNDLE_METADATA');
  if (Object.hasOwn(plist,'LSMinimumSystemVersion') &&
      (typeof plist.LSMinimumSystemVersion !== 'string' || !version.test(plist.LSMinimumSystemVersion)))
    throw Error('INVALID_BUNDLE_METADATA');
  return {bundleId:plist.CFBundleIdentifier, executable:plist.CFBundleExecutable,
    version:plist.CFBundleShortVersionString, build:plist.CFBundleVersion,
    minimumMacOS:plist.LSMinimumSystemVersion ?? null,
    multipleInstancesProhibited:plist.LSMultipleInstancesProhibited ?? null,
    requiresNativeExecution:plist.LSRequiresNativeExecution ?? null};
}

export function architectureList(output) {
  const items=output.trim().split(/\s+/);
  if (!items.length || items.length>4 || new Set(items).size!==items.length ||
      items.some(item=>!['arm64','arm64e','x86_64','x86_64h'].includes(item)))
    throw Error('UNREVIEWED_ARCHITECTURE');
  return items.sort();
}

export function entitlementSummary(plist) {
  if (plist === null) return {appSandbox:null,appGroupsPresent:null,keychainGroupsPresent:null};
  if (!dictionary(plist)) throw Error('INVALID_ENTITLEMENTS');
  const sandbox=plist['com.apple.security.app-sandbox'];
  if (sandbox!==undefined && typeof sandbox!=='boolean') throw Error('INVALID_ENTITLEMENTS');
  const group = key => {
    if (!Object.hasOwn(plist,key)) return false;
    if (!Array.isArray(plist[key]) || plist[key].some(value=>typeof value!=='string'))
      throw Error('INVALID_ENTITLEMENTS');
    return plist[key].length>0;
  };
  return {appSandbox:sandbox??false,
    appGroupsPresent:group('com.apple.security.application-groups'),
    keychainGroupsPresent:group('keychain-access-groups')};
}

function run(tool,args,input) {
  const result=spawnSync('/usr/bin/'+tool,args,{input,encoding:'utf8',timeout:90_000,
    maxBuffer:256*1024,env:{PATH:'/usr/bin:/bin',LC_ALL:'C'},shell:false});
  if (result.error || result.status!==0) throw Error('ARTIFACT_TOOL_FAILED:'+tool);
  return {stdout:result.stdout,stderr:result.stderr};
}
function fingerprint(path,kind) {
  const stat=lstatSync(path,{bigint:true});
  if (realpathSync(path)!==path || (kind==='directory'?!stat.isDirectory():!stat.isFile()) ||
      (kind==='file' && (stat.size===0n || stat.size>2_147_483_648n)))
    throw Error('INVALID_ARTIFACT_PATH');
  return [stat.dev,stat.ino,stat.size,stat.mtimeNs,stat.ctimeNs].join(':');
}
async function fileHash(path) {
  const hash=createHash('sha256');
  for await (const chunk of createReadStream(path)) hash.update(chunk);
  return hash.digest('hex');
}
function signingMetadata(output,expected) {
  const id=output.match(/^Identifier=([A-Za-z0-9.-]+)$/m)?.[1];
  const team=output.match(/^TeamIdentifier=([A-Z0-9]{10})$/m)?.[1];
  const cdhash=output.match(/^CDHash=([0-9a-f]{40})$/m)?.[1];
  if (id!==expected || !team || !cdhash) throw Error('UNREVIEWED_SIGNING_METADATA');
  return {identifier:id,teamIdentifier:team,cdHash:cdhash};
}

export async function inspectBundle(path) {
  if (process.platform!=='darwin') throw Error('MACOS_REQUIRED');
  if (typeof path!=='string' || !isAbsolute(path) || !path.endsWith('.app'))
    throw Error('EXPLICIT_APP_BUNDLE_REQUIRED');
  const root=resolve(path);
  const plist=join(root,'Contents','Info.plist');
  const fixed=[[root,'directory'],[plist,'file']];
  const before=fixed.map(([name,kind])=>fingerprint(name,kind));
  const metadata=bundleMetadata(JSON.parse(run('plutil',['-convert','json','-o','-',plist]).stdout));
  const exe=join(root,'Contents','MacOS',metadata.executable);
  fixed.push([exe,'file']);
  before.push(fingerprint(exe,'file'));
  const architectures=architectureList(run('lipo',['-archs',exe]).stdout);
  const executableSha256=await fileHash(exe);
  run('codesign',['--verify','--deep','--strict','--all-architectures',root]);
  const signatures=architectures.map(arch=>{
    const display=run('codesign',['--display','--arch',arch,'--verbose=4',root]);
    const requirement=run('codesign',['--display','--arch',arch,'-r','-',root]);
    const designated=requirement.stderr.match(/^designated => (.+)$/m)?.[1];
    if (!designated) throw Error('DESIGNATED_REQUIREMENT_UNAVAILABLE');
    const ent=run('codesign',['--display','--arch',arch,'--entitlements','-','--xml',root]).stdout.trim();
    const values=ent?JSON.parse(run('plutil',['-convert','json','-o','-','-'],ent).stdout):null;
    return {arch,...signingMetadata(display.stderr,metadata.bundleId),
      designatedRequirementSha256:createHash('sha256').update(designated).digest('hex'),
      ...entitlementSummary(values)};
  });
  if (fixed.some(([name,kind],index)=>fingerprint(name,kind)!==before[index]))
    throw Error('ARTIFACT_CHANGED');
  return {schema:1,status:'STATIC_METADATA_ONLY',...metadata,architectures,executableSha256,
    signatureIntegrity:'CODESIGN_VERIFIED',signatures,
    officialPublisher:'NOT_PINNED',distributionPackage:'NOT_VERIFIED',
    profileIsolation:'NOT_VERIFIED',runtimeAdmission:false};
}

// Offline Mach-O inspection from Windows or macOS. Bounded reads and
// a streaming-sized hash buffer; no execution, network, or trust decision.
// Layout constants: Apple's XNU loader.h, fat.h, cs_blobs.h.
const MAX_BINARY = 2_147_483_648;
const MAX_COMMANDS = 4 * 1024 * 1024;
const MAX_SIGNATURE = 32 * 1024 * 1024;
const bad = () => { throw Error('UNREVIEWED_MACH_O'); };
const within = (offset, size, bound) => Number.isSafeInteger(offset) &&
  Number.isSafeInteger(size) && offset >= 0 && size >= 0 && offset <= bound && size <= bound - offset;
const be = (b, p) => b.readUInt32BE(p);
const le = (b, p) => b.readUInt32LE(p);

function archName(cpu, subtype) {
  const sub = subtype & 0x00ffffff;
  if (cpu===0x01000007 && (sub===3 || sub===8)) return sub===8?'x86_64h':'x86_64';
  if (cpu===0x0100000c && (sub===0 || sub===1 || sub===2)) return sub===2?'arm64e':'arm64';
  return bad();
}
function boundedString(bytes, offset, minimum, limit=255) {
  if (!within(offset,1,bytes.length) || offset<minimum) return bad();
  const end=bytes.indexOf(0,offset);
  if (end<0 || end-offset>limit) return bad();
  return bytes.subarray(offset,end).toString('utf8');
}

// Reader accepts a bounded file-relative read, also usable by synthetic tests.
export function machOSummary(read, size) {
  if (!Number.isSafeInteger(size) || size<32 || size>MAX_BINARY) return bad();
  const get=(offset,length)=>{
    if (!within(offset,length,size)) return bad();
    const bytes=read(offset,length);
    if (!Buffer.isBuffer(bytes)||bytes.length!==length) return bad();
    return bytes;
  };
  const head=get(0,8), magic=be(head,0);
  let slices=[];
  if (magic===0xcafebabe || magic===0xcafebabf) {
    const count=be(head,4), wide=magic===0xcafebabf, stride=wide?32:20;
    if (count<1 || count>4) return bad();
    const table=get(8,count*stride);
    for(let i=0;i<count;i++){
      const p=i*stride;
      const offset=wide?Number(table.readBigUInt64BE(p+8)):be(table,p+8);
      const length=wide?Number(table.readBigUInt64BE(p+16)):be(table,p+12);
      if(offset<8+count*stride || length<32 || !within(offset,length,size))return bad();
      slices.push({offset,length,cpu:be(table,p),subtype:be(table,p+4)});
    }
    const ordered=[...slices].sort((a,b)=>a.offset-b.offset);
    for(let i=1;i<ordered.length;i++)
      if(ordered[i].offset<ordered[i-1].offset+ordered[i-1].length)return bad();
  } else if (magic===0xcffaedfe) {
    slices=[{offset:0,length:size}];
  } else return bad(); // Unsupported byte order and 32-bit formats refuse.

  const results=slices.map(slice=>{
    const h=get(slice.offset,32);
    if(le(h,0)!==0xfeedfacf || le(h,12)!==2) return bad(); // MH_EXECUTE
    const cpu=le(h,4),subtype=le(h,8);
    if(slice.cpu!==undefined&&(slice.cpu!==cpu||slice.subtype!==subtype))return bad();
    const arch=archName(cpu,subtype),count=le(h,16),length=le(h,20);
    if(count>65536||length>MAX_COMMANDS||!within(32,length,slice.length))return bad();
    const commands=get(slice.offset+32,length);
    let position=0,signature=null;
    const indicators=new Set();
    const names=['OneAuth','MSAL','Security','AuthenticationServices','WebKit','AppKit'];
    for(let i=0;i<count;i++){
      if(!within(position,8,length))return bad();
      const cmd=le(commands,position),n=le(commands,position+4);
      if(n<8||n%8!==0||!within(position,n,length))return bad();
      const bytes=commands.subarray(position,position+n);
      if(cmd===0x1d){
        if(n!==16||signature!==null)return bad();
        const offset=le(bytes,8),length=le(bytes,12);
        if(length<12||length>MAX_SIGNATURE||offset<32+commands.length||
          !within(offset,length,slice.length))return bad();
        signature={offset,length};
      } else if([0xc,0x80000018,0x8000001f,0x20,0x80000023].includes(cmd)){
        if(n<24)return bad();
        const name=boundedString(bytes,le(bytes,8),24,4096);
        for(const hint of names) {
          if(name.toLowerCase().includes(hint.toLowerCase()))indicators.add(hint);
        }
      }
      position+=n;
    }
    if(position!==length)return bad();
    if(!signature)return {arch,linkedLibraryIndicators:[...indicators].sort(),embeddedSignature:null};
    const start=slice.offset+signature.offset;
    const hdr=get(start,12),blobLength=be(hdr,4),slots=be(hdr,8);
    if(be(hdr,0)!==0xfade0cc0||slots<1||slots>128||
      !within(12,slots*8,blobLength)||blobLength>signature.length)return bad();
    const index=get(start+12,slots*8);
    const seen=new Set(),ranges=[],directories=[];
    let cmsBlobPresent=false,derEntitlementsPresent=false,xmlEntitlementKeyIndicators=null;
    for(let i=0;i<slots;i++){
      const type=be(index,i*8),offset=be(index,i*8+4);
      if(seen.has(type)||offset<12+slots*8||!within(offset,8,blobLength))return bad();
      seen.add(type);
      const prefix=get(start+offset,8),magic=be(prefix,0),n=be(prefix,4);
      if(n<8||!within(offset,n,blobLength))return bad();
      ranges.push({offset,length:n});
      if(type===0||(type>=0x1000&&type<0x1005)){
        if(magic!==0xfade0c02||n<44)return bad();
        const d=get(start+offset,Math.min(n,65536));
        const ver=be(d,8),flags=be(d,12);
        const min=ver>=0x20200?52:ver>=0x20100?48:44;
        if(d.length<min)return bad();
        const identifier=boundedString(d,be(d,20),min);
        const teamOffset=ver>=0x20200?be(d,48):0;
        const team=teamOffset?boundedString(d,teamOffset,min):null;
        // Expose only expected product/team comparisons, never arbitrary strings.
        directories.push({slot:type,expectedIdentifier:identifier===expectedBundleId,
          expectedTeam:team===null?null:team==='UBF8T346G9',
          hardenedRuntimeFlag:(flags&0x10000)!==0,adHocFlag:(flags&2)!==0});
      } else if(type===5){
        if(magic!==0xfade7171||n>256*1024)return bad();
        const xml=get(start+offset+8,n-8).toString('utf8');
        if(!/<plist(?:\s|>)/.test(xml))return bad();
        // Lexical indications only, not XML semantic or effective entitlement proof.
        xmlEntitlementKeyIndicators={};
        for(const key of ['com.apple.security.app-sandbox','com.apple.security.application-groups',
          'keychain-access-groups','com.apple.security.cs.disable-library-validation',
          'com.apple.security.cs.allow-dyld-environment-variables']){
          xmlEntitlementKeyIndicators[key]=xml.includes('<key>'+key+'</key>');
        }
      } else if(type===7){
        if(magic!==0xfade7172)return bad();
        derEntitlementsPresent=true;
      } else if(type===0x10000){
        if(magic!==0xfade0b01)return bad();
        cmsBlobPresent=true;
      }
    }
    ranges.sort((a,b)=>a.offset-b.offset);
    for(let i=1;i<ranges.length;i++)
      if(ranges[i].offset<ranges[i-1].offset+ranges[i-1].length)return bad();
    if(!directories.some(d=>d.slot===0))return bad();
    return {arch,linkedLibraryIndicators:[...indicators].sort(),embeddedSignature:{
      codeDirectories:directories,cmsBlobPresent,derEntitlementsPresent,xmlEntitlementKeyIndicators}};
  });
  if(new Set(results.map(s=>s.arch)).size!==results.length)return bad();
  return results.sort((a,b)=>a.arch.localeCompare(b.arch));
}

export function inspectMachO(path) {
  if(typeof path!=='string'||!isAbsolute(path)||basename(path)!==expectedExecutable)
    throw Error('EXPLICIT_EXECUTABLE_REQUIRED');
  const root=resolve(path);
  const before=fingerprint(root,'file');
  const fd=openSync(root,constants.O_RDONLY|(constants.O_NOFOLLOW??0)|(constants.O_NONBLOCK??0));
  try {
    const initial=fstatSync(fd,{bigint:true});
    const identity=s=>[s.dev,s.ino,s.size,s.mtimeNs,s.ctimeNs].join(':');
    if(!initial.isFile()||identity(initial)!==before||initial.size>BigInt(MAX_BINARY))return bad();
    const size=Number(initial.size);
    const read=(offset,length)=>{
      if(!within(offset,length,size))return bad();
      const b=Buffer.alloc(length);
      let done=0;
      while(done<length){
        const n=readSync(fd,b,done,length-done,offset+done);
        if(n===0)return bad();
        done+=n;
      }
      return b;
    };
    const slices=machOSummary(read,size);
    const hash=createHash('sha256');
    for(let pos=0;pos<size;pos+=1024*1024)
      hash.update(read(pos,Math.min(1024*1024,size-pos)));
    if(identity(fstatSync(fd,{bigint:true}))!==before||fingerprint(root,'file')!==before)
      throw Error('ARTIFACT_CHANGED');
    return {schema:1,status:'OFFLINE_MACH_O_METADATA_ONLY',sizeBytes:size,executableSha256:hash.digest('hex'),
      slices,signatureIntegrity:'NOT_VERIFIED',officialPublisher:'NOT_PINNED',
      distributionPackage:'NOT_VERIFIED',profileIsolation:'NOT_VERIFIED',runtimeAdmission:false};
  } finally {closeSync(fd);}
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href===import.meta.url) {
  try {
    if(process.argv.length===4 && process.argv[2]==='--mach-o')
      console.log(JSON.stringify(inspectMachO(process.argv[3]),null,2));
    else {
      if (process.argv.length!==3) throw Error('EXPLICIT_APP_BUNDLE_REQUIRED');
      console.log(JSON.stringify(await inspectBundle(process.argv[2]),null,2));
    }
  } catch {
    // Native tool diagnostics may contain private host paths. Do not print them.
    console.error(JSON.stringify({schema:1,status:'INSPECTION_BLOCKED',runtimeAdmission:false}));
    process.exitCode=1;
  }
}
