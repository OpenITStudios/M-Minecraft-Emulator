import {invokeRuntime} from './invokeRuntime.js';

export interface ProcessKey {pid:number;createdFileTime:string}
export interface ContextComparison {sameWindowsUser:boolean;sameProfileDirectory:boolean;sameWindowsSession:boolean}

export function parseContextComparison(value:unknown):ContextComparison {
  if(!value||typeof value!=='object'||Array.isArray(value))throw new TypeError('Invalid Windows context comparison');
  const result=value as Record<string,unknown>;
  const fields=['sameWindowsUser','sameProfileDirectory','sameWindowsSession'];
  if(Object.keys(result).length!==fields.length||fields.some(field=>typeof result[field]!=='boolean'))throw new TypeError('Invalid Windows context comparison');
  return result as unknown as ContextComparison;
}

export async function compareProcessContexts(left:ProcessKey,right:ProcessKey):Promise<ContextComparison> {
  const key=(process:ProcessKey)=>{
    if(!Number.isInteger(process.pid)||process.pid<=0||process.pid>0xffffffff||!/^\d{1,20}$/.test(process.createdFileTime)||BigInt(process.createdFileTime)===0n||BigInt(process.createdFileTime)>0xffffffffffffffffn)throw new TypeError('Invalid Windows process identity');
    return {pid:process.pid,createdFileTime:process.createdFileTime};
  };
  return parseContextComparison(await invokeRuntime('desktop_compare_process_contexts',{left:key(left),right:key(right)}));
}
