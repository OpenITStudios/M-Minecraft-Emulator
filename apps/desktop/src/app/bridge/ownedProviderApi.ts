import {invokeRuntime} from "./invokeRuntime.js";
export type OwnedOperation = "Inspect" | "Create" | "Launch" | "Focus" | "PlaceLeft" | "PlaceRight" | "Close";
export type ClientId = "01" | "02" | "03" | "04";
export interface OwnedProcess {pid:number;createdFileTime:string;sessionId:number;windowHandle:string;responding:boolean}
export interface OwnedSlot {clientId:Exclude<ClientId,"01">;box:string;profileExists:boolean;settingsVerified:boolean;boxedProcessCount:number;launchAllowed:boolean;status:"ABSENT"|"BLOCKED"|"STOPPED"|"STARTING_OR_CLOSING"|"RUNNING";processes:OwnedProcess[]}
export interface OwnedSnapshot {schema:1;provider:"OWNED_V1186_LOCAL_TEST";operation:OwnedOperation;launcherExitCode:number|null;focusError:string|null;clients:OwnedSlot[];nativeProcesses:OwnedProcess[]}
const record=(v:unknown):v is Record<string,unknown>=>typeof v==="object" && v!==null && !Array.isArray(v);
const keys=(v:Record<string,unknown>, names:string[])=>Object.keys(v).length===names.length && names.every(n=>Object.hasOwn(v,n));
export function parseOwnedSnapshot(value:unknown):OwnedSnapshot {
  if(!record(value)||!keys(value,["schema","provider","operation","launcherExitCode","focusError","clients","nativeProcesses"])||value.schema!==1||value.provider!=="OWNED_V1186_LOCAL_TEST"||
    !["Inspect","Create","Launch","Focus","PlaceLeft","PlaceRight","Close"].includes(String(value.operation))||
    !(value.launcherExitCode===null||Number.isInteger(value.launcherExitCode))||!(value.focusError===null||(typeof value.focusError==="string"&&value.focusError.length<=2048))||!Array.isArray(value.clients)||value.clients.length!==3||!Array.isArray(value.nativeProcesses)||value.nativeProcesses.length>1) throw new TypeError("Untrusted owned provider observation.");
  const pids=new Set<number>();
  const process=(p:unknown,requireWindow=false)=>{
    if(!record(p)||!keys(p,["pid","createdFileTime","sessionId","windowHandle","responding"])||!Number.isInteger(p.pid)||Number(p.pid)<=0||Number(p.pid)>0xffffffff||pids.has(Number(p.pid))||
      typeof p.createdFileTime!=="string"||!/^[1-9][0-9]{0,19}$/.test(p.createdFileTime)||BigInt(p.createdFileTime)>0xffffffffffffffffn||
      !Number.isInteger(p.sessionId)||Number(p.sessionId)<0||Number(p.sessionId)>0xffffffff||typeof p.windowHandle!=="string"||!/^0x[0-9a-f]{1,16}$/.test(p.windowHandle)||
      (requireWindow&&BigInt(p.windowHandle)===0n)||typeof p.responding!=="boolean") throw new TypeError("Untrusted process identity.");
    pids.add(Number(p.pid));
  };
  value.nativeProcesses.forEach(p=>process(p));
  value.clients.forEach((s:unknown,index:number)=>{
    const id=String(index+2).padStart(2,"0");
    if(!record(s)||!keys(s,["clientId","box","profileExists","settingsVerified","boxedProcessCount","launchAllowed","status","processes"])||s.clientId!==id||s.box!==`MivubiEdu${id}`||
      typeof s.launchAllowed!=="boolean"||(s.launchAllowed&&(!s.settingsVerified||s.boxedProcessCount!==0))||typeof s.profileExists!=="boolean"||typeof s.settingsVerified!=="boolean"||!Number.isInteger(s.boxedProcessCount)||Number(s.boxedProcessCount)<0||Number(s.boxedProcessCount)>256||
      !["ABSENT","BLOCKED","STOPPED","STARTING_OR_CLOSING","RUNNING"].includes(String(s.status))||!Array.isArray(s.processes)||s.processes.length>1||s.processes.length>Number(s.boxedProcessCount)||
      (s.settingsVerified&&!s.profileExists)||(s.status==="ABSENT"&&(s.profileExists||s.boxedProcessCount!==0))||(s.status==="STOPPED"&&(!s.settingsVerified||s.boxedProcessCount!==0))||
      (s.status==="RUNNING"&&(!s.settingsVerified||s.processes.length!==1))) throw new TypeError("Inconsistent owned slot observation.");
    s.processes.forEach(p=>process(p,s.status==="RUNNING"));
  });
  return value as unknown as OwnedSnapshot;
}
export const ownedProvider={
  selectSession:()=>invokeRuntime<string|null>("desktop_select_owned_test_session"),
  execute:async(sessionDirectory:string,clientId:ClientId,operation:OwnedOperation,licensed=false,compatibilityConfirmed=false,target:OwnedProcess|null=null)=>{
    const snapshot=parseOwnedSnapshot(await invokeRuntime<unknown>("desktop_owned_test_operation",{request:{sessionDirectory,clientId,operation,licensed,compatibilityConfirmed,target}}));
    if(snapshot.operation!==operation)throw new TypeError("Mismatched owned operation acknowledgement.");
    return snapshot;
  },
};
