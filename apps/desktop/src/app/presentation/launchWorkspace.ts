import type {ClientId,OwnedProcess,OwnedSnapshot} from '../bridge/ownedProviderApi.js';

/** Observe a single launch; never start another game while waiting for its window. */
export async function observeWorkspaceLaunch(
  client:ClientId,
  inspect:()=>Promise<OwnedSnapshot>,
  present:((client:ClientId,target:OwnedProcess)=>Promise<void>)|null,
  publish:(snapshot:OwnedSnapshot)=>void,
  pause:()=>Promise<void>=()=>new Promise(resolve=>setTimeout(resolve,1500)),
):Promise<boolean>{
  for(let attempt=0;attempt<8;attempt++){
    const snapshot=await inspect();publish(snapshot);
    const slot=snapshot.clients.find(slot=>slot.clientId===client);
    const processes=client==='01'?snapshot.nativeProcesses:slot?.status==='RUNNING'?slot.processes:[];
    const target=processes?.length===1?processes[0]:null;
    if(target&&target.windowHandle!=='0x0'&&target.responding){
      await present?.(client,target);return true;
    }
    if(attempt<7)await pause();
  }
  return false;
}
