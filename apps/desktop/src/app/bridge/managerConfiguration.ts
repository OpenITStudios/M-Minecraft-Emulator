import {invokeRuntime} from './invokeRuntime.js';
export interface ManagerSettings {sessionDirectory:string;minecraftExecutable:string}
export interface MinecraftTarget {id:'01'|'02'|'03'|'04';kind:'NATIVE'|'ISOLATED';provider:string;profile:string|null}
export interface ManagerConfiguration {detectedMinecraft:string|null;targetError:string|null;settings:ManagerSettings;catalog:{schema:1;targets:MinecraftTarget[]}}
export function parseManagerConfiguration(value:unknown):ManagerConfiguration {
  const v=value as ManagerConfiguration;
  if(!v||!(v.detectedMinecraft===null||typeof v.detectedMinecraft==='string')||!(v.targetError===null||typeof v.targetError==='string')||!v.settings||typeof v.settings.sessionDirectory!=='string'||typeof v.settings.minecraftExecutable!=='string'||v.catalog?.schema!==1||!Array.isArray(v.catalog.targets)||v.catalog.targets.length!==4)throw new TypeError('Invalid manager configuration');
  const profiles=new Set<string>();
  v.catalog.targets.forEach((t,i)=>{
    if(t.id!==String(i+1).padStart(2,'0')||t.kind!==(i===0?'NATIVE':'ISOLATED')||t.provider!==(i===0?'OFFICIAL_WINDOWS':'OWNED_V1186')||(i===0?t.profile!==null:typeof t.profile!=='string'||t.profile!==`MivubiEdu${t.id}`||profiles.has(t.profile)))throw new TypeError('Invalid target catalog');
    if(t.profile)profiles.add(t.profile);
  });
  return v;
}
export const managerConfiguration={load:async()=>parseManagerConfiguration(await invokeRuntime('desktop_manager_settings',{settings:null})),save:async(settings:ManagerSettings)=>parseManagerConfiguration(await invokeRuntime('desktop_manager_settings',{settings}))};
export function operationIssue(error:unknown){
  if(error && typeof error==='object' && 'code' in error && 'message' in error && 'action' in error && 'details' in error && typeof error.message==='string' && typeof error.action==='string' && typeof error.details==='string')return {message:error.message,action:error.action,details:error.details};
  const details=(error instanceof Error?error.message:typeof error==='string'?error:JSON.stringify(error))??'Unknown error';
  if(/Recent launch|Previous private launcher/.test(details))return {message:'A previous launch is still pending.',action:'Check again before retrying. The launch guard also requires 30 seconds.',details};
  if(/signature|hash|payload|service|driver|ABI|Test Mode/i.test(details))return {message:'The selected engine could not be verified.',action:'Check engine configuration in Settings. No fallback engine was used.',details};
  if(/Minecraft|executable|installation/i.test(details))return {message:'Minecraft could not be opened or verified.',action:'Check its installed location in Settings, then check the clients again.',details};
  return {message:'The operation could not be completed.',action:'Check again to obtain a fresh status. Open Diagnostics for the technical details.',details};
}
