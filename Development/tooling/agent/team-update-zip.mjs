// Bounded read-only ZIP inspector for operator-supplied team module snapshots.
// Not a generic ZIP extractor: never writes extracted contents or executes code.
// Only supports regular ZIP32 with STORE/DEFLATE, no encrypted or linked entries.
import {createHash} from 'node:crypto';
import {lstat,open} from 'node:fs/promises';
import {inflateRawSync} from 'node:zlib';
import path from 'node:path';

const MAX_ARCHIVE_BYTES=64*1024*1024;  // Skills Probe fixture ~38 MiB
const MAX_ENTRIES=2500;
const MAX_CENTRAL=2*1024*1024;
const MAX_MEMBER=128*1024;
const MAX_NAME_BYTES=300;
const READ_CHUNK=128*1024;
const SHA=/^[a-f0-9]{64}$/;
function failure(){throw new Error('UNSUPPORTED_OR_UNSAFE_TEAM_ZIP');}
function crc32(buffer){
  let c=0xffffffff;
  for(let i=0;i<buffer.length;i++){
    c^=buffer[i];
    for(let j=0;j<8;j++)c=(c>>>1)^((c&1)?0xedb88320:0);
  }
  return (c^0xffffffff)>>>0;
}
function same(a,b){
  return a.dev===b.dev&&a.ino!==0n&&a.ino===b.ino&&
    a.size===b.size&&a.mtimeNs===b.mtimeNs&&a.ctimeNs===b.ctimeNs;
}
async function readAt(handle,position,length){
  if(!Number.isSafeInteger(length)||length<0||
     !Number.isSafeInteger(position)||position<0)failure();
  const bytes=Buffer.alloc(length);
  let read=0;
  while(read<length){
    const {bytesRead}=await handle.read(bytes,read,length-read,position+read);
    if(bytesRead<=0||bytesRead>length-read)failure();
    read+=bytesRead;
  }
  return bytes;
}
function safeName(name){
  return name.length>0&&name.length<=MAX_NAME_BYTES&&
    !name.includes('\0')&&!name.includes('\\')&&
    !name.startsWith('/')&&!name.startsWith('./')&&
    !name.includes('//')&&!name.split('/').includes('..')&&
    !name.split('/').includes('.');
}
export async function openTeamZip(filename){
  if(typeof filename!=='string'||!path.isAbsolute(filename)||
     path.extname(filename).toLowerCase()!=='.zip')failure();
  const before=await lstat(filename,{bigint:true});
  if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1n||
     before.size<22n||before.size>BigInt(MAX_ARCHIVE_BYTES))failure();
  const handle=await open(filename,'r');
  try{
    const pinned=await handle.stat({bigint:true});
    if(!same(before,pinned))failure();
    const size=Number(pinned.size);
    const tailLength=Math.min(size,22+65535);
    const tail=await readAt(handle,size-tailLength,tailLength);
    let eocd=-1;
    for(let offset=tail.length-22;offset>=0;offset--){
      if(tail.readUInt32LE(offset)===0x06054b50 &&
         offset+22+tail.readUInt16LE(offset+20)===tail.length){
        eocd=offset;break;
      }
    }
    if(eocd<0||tail.readUInt16LE(eocd+4)!==0||
       tail.readUInt16LE(eocd+6)!==0)failure(); // no spanning/multi-disk
    const count=tail.readUInt16LE(eocd+10);
    const onDisk=tail.readUInt16LE(eocd+8);
    const centralSize=tail.readUInt32LE(eocd+12);
    const centralOffset=tail.readUInt32LE(eocd+16);
    if(count===0||count>MAX_ENTRIES||onDisk!==count||
       count===0xffff||centralSize===0xffffffff||
       centralOffset===0xffffffff||centralSize>MAX_CENTRAL||
       centralOffset+centralSize>size-tailLength+eocd)failure();
    const central=await readAt(handle,centralOffset,centralSize);
    const entries=new Map();
    let cursor=0;
    for(let i=0;i<count;i++){
      if(cursor+46>central.length||central.readUInt32LE(cursor)!==0x02014b50)
        failure();
      const flags=central.readUInt16LE(cursor+8);
      const method=central.readUInt16LE(cursor+10);
      const crc=central.readUInt32LE(cursor+16);
      const packed=central.readUInt32LE(cursor+20);
      const original=central.readUInt32LE(cursor+24);
      const nameLen=central.readUInt16LE(cursor+28);
      const extraLen=central.readUInt16LE(cursor+30);
      const commentLen=central.readUInt16LE(cursor+32);
      const offset=central.readUInt32LE(cursor+42);
      const ext=central.readUInt32LE(cursor+38);
      const sizeEntry=46+nameLen+extraLen+commentLen;
      if(cursor+sizeEntry>central.length||nameLen<1||
         nameLen>MAX_NAME_BYTES||packed===0xffffffff||
         original===0xffffffff||offset===0xffffffff||
         flags&1||![0,8].includes(method))failure();
      const name=central.subarray(cursor+46,cursor+46+nameLen).toString('utf8');
      const mode=(ext>>>16)&0xf000;
      if(!safeName(name)||mode===0xa000||entries.has(name))failure();
      if(!name.endsWith('/')) {
        entries.set(name,{name,flags,method,crc,packed,original,offset});
      }
      cursor+=sizeEntry;
    }
    if(cursor!==central.length)failure();
    async function readMember(name){
      const entry=entries.get(name);
      if(!entry||entry.original>MAX_MEMBER||entry.packed>MAX_MEMBER||
         entry.offset+30>centralOffset)failure();
      const header=await readAt(handle,entry.offset,30);
      if(header.readUInt32LE(0)!==0x04034b50||
         header.readUInt16LE(6)!==entry.flags||
         header.readUInt16LE(8)!==entry.method)failure();
      const nameLength=header.readUInt16LE(26);
      const extraLength=header.readUInt16LE(28);
      if(nameLength!==Buffer.byteLength(name,'utf8'))failure();
      const localName=await readAt(handle,entry.offset+30,nameLength);
      if(!localName.equals(Buffer.from(name,'utf8')))failure();
      const position=entry.offset+30+nameLength+extraLength;
      if(position+entry.packed>centralOffset)failure();
      const zipped=await readAt(handle,position,entry.packed);
      let unpacked;
      try{
        unpacked=entry.method===0?zipped:
          inflateRawSync(zipped,{maxOutputLength:MAX_MEMBER});
      }catch{failure();}
      if(unpacked.length!==entry.original||crc32(unpacked)!==entry.crc)
        failure();
      return unpacked;
    }
    async function archiveDigest(){
      const hash=createHash('sha256');
      for(let offset=0;offset<size;offset+=READ_CHUNK){
        hash.update(await readAt(handle,offset,Math.min(READ_CHUNK,size-offset)));
      }
      return hash.digest('hex');
    }
    async function verifyStable(){
      const after=await handle.stat({bigint:true});
      const named=await lstat(filename,{bigint:true});
      if(!same(pinned,after)||!same(pinned,named))failure();
    }
    return {
      entries:new Set(entries.keys()),
      readMember,archiveDigest,verifyStable,
      close:()=>handle.close(),
    };
  }catch(e){await handle.close();throw e;}
}
export function sha256(bytes){
  const output=createHash('sha256').update(bytes).digest('hex');
  if(!SHA.test(output))failure();
  return output;
}
