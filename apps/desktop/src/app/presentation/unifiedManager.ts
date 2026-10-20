import type {OwnedSlot,OwnedOperation} from '../bridge/ownedProviderApi.js';
export function ownedClientStatus(slot:OwnedSlot,current:boolean,pending:boolean,active:OwnedOperation|null,closing=false):string {
  if(active)return active==='Launch'?'Starting':active==='Close'?'Closing':active==='Create'?'Setting up':'Checking';
  if(!current)return 'Needs attention';
  if(closing&&slot.processes.length)return 'Closing';
  if(slot.status==='ABSENT')return 'Not configured';
  if(slot.status==='RUNNING')return 'Running';
  if(slot.status==='STARTING_OR_CLOSING')return 'Starting / closing';
  if(slot.status==='STOPPED'&&!pending&&slot.launchAllowed)return 'Ready';
  return 'Needs attention';
}
