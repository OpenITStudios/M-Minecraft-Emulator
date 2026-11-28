export interface OwnedProcess {pid:number;createdFileTime:string;sessionId:number;windowHandle:string;responding:boolean}
// Compare validated observations only. Neither equality nor this transient
// key is native ownership, liveness or permission to manipulate a window.
export function sameWindowIdentity(a:OwnedProcess,b:OwnedProcess):boolean {
  return a.pid===b.pid && a.createdFileTime===b.createdFileTime &&
    a.sessionId===b.sessionId && a.windowHandle===b.windowHandle;
}
export function windowIdentityKey(process:OwnedProcess|null|undefined):string {
  return process ? [process.pid,process.createdFileTime,process.sessionId,process.windowHandle].join('/') : '';
}
const record=(v:unknown):v is Record<string,unknown>=>typeof v==="object" && v!==null && !Array.isArray(v);
const keys=(v:Record<string,unknown>, names:string[])=>Object.keys(v).length===names.length && names.every(n=>Object.hasOwn(v,n));
export function parseObservedProcess(p:unknown):OwnedProcess {
  if(!record(p)||!keys(p,["pid","createdFileTime","sessionId","windowHandle","responding"])||
    !Number.isInteger(p.pid)||Number(p.pid)<=0||Number(p.pid)>0xffffffff||
    typeof p.createdFileTime!=="string"||!/^[1-9][0-9]{0,19}$/.test(p.createdFileTime)||
    BigInt(p.createdFileTime)>0xffffffffffffffffn||
    !Number.isInteger(p.sessionId)||Number(p.sessionId)<0||Number(p.sessionId)>0xffffffff||
    typeof p.windowHandle!=="string"||!/^0x[0-9a-f]{1,16}$/.test(p.windowHandle)||
    typeof p.responding!=="boolean")throw new TypeError("Untrusted process identity.");
  return p as unknown as OwnedProcess;
}
