import type {OwnedSlot,OwnedOperation,OwnedSnapshot,ClientId} from '../bridge/ownedProviderApi.js';
export function clientActionBlockedReason(snapshot:Pick<OwnedSnapshot,'nativeProcesses'|'clients'>,client:ClientId,current:boolean,pending:boolean,closing:boolean):string|null {
  if(!current)return 'Check all clients to refresh the status.';
  if(closing)return 'Minecraft is closing. Check all clients before reopening.';
  const slot=snapshot.clients.find(slot=>slot.clientId===client);
  const processes=client==='01'?snapshot.nativeProcesses:slot?.processes??[];
  if(processes.length>1)return 'Check all clients: multiple game windows were detected.';
  if(processes.length===1){
    if(!processes[0].responding||processes[0].windowHandle==='0x0')return 'The game window is not responding. Check all clients.';
    if(client!=='01'&&(!slot?.settingsVerified||slot.status!=='RUNNING'))return 'Check all clients to verify this private session.';
    return null;
  }
  if(pending)return 'A previous launch is pending. Check all clients before retrying.';
  if(client==='01')return null;
  if(!slot||slot.boxedProcessCount!==0)return 'Check all clients before opening this session.';
  if(!slot.profileExists)return null;
  return slot.settingsVerified&&slot.launchAllowed?null:'This session needs a fresh check before opening.';
}
export function ownedClientStatus(slot:OwnedSlot,current:boolean,pending:boolean,active:OwnedOperation|null,closing=false):string {
  if(active)return active==='Launch'?'Starting':active==='Close'?'Closing':active==='Create'?'Setting up':'Checking';
  if(!current)return 'Needs attention';
  if(closing&&slot.processes.length)return 'Closing';
  if(slot.status==='ABSENT')return 'Not opened';
  if(slot.status==='RUNNING')return slot.processes.length===1&&slot.processes[0].responding&&slot.processes[0].windowHandle!=='0x0'?'Running':'Needs attention';
  if(slot.status==='STARTING_OR_CLOSING')return 'Starting / closing';
  if(slot.status==='STOPPED'&&!pending&&slot.launchAllowed)return 'Not opened';
  return 'Needs attention';
}
