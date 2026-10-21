<script lang="ts">
  import {ownedProvider,type OwnedSnapshot,type OwnedOperation,type ClientId,type OwnedProcess} from "./app/bridge/ownedProviderApi.js";
  import {ownedClientStatus,clientActionBlockedReason} from "./app/presentation/unifiedManager.js";
  import {operationIssue} from "./app/bridge/managerConfiguration.js";
  import {nativeFeasibility} from "./app/bridge/nativeFeasibilityApi.js";
  import {observeWorkspaceLaunch} from './app/presentation/launchWorkspace.js';
  export let sessionDirectory:string;
  export let initialSnapshot:OwnedSnapshot;
  export let operationBusy=false;
  export let externalBusy=false;
  export let tabStates:Record<ClientId,string>={"01":"Checking","02":"Checking","03":"Checking","04":"Checking"};
  export let onPresent:((client:ClientId,target:OwnedProcess)=>Promise<void>)|null=null;
  export let onRelease:(()=>Promise<void>)|null=null;
  export let onResume:(()=>Promise<void>)|null=null;
  export let onObserved:((snapshot:OwnedSnapshot)=>Promise<void>)|null=null;
  let snapshot=initialSnapshot;
  let error="",notice="",details="",activeClient="",activeOperation="";
  let licensed=false,compatibility=false;
  let setupDialog:HTMLDialogElement;
  let setupClient:ClientId|null=null;
  async function openClient(client:ClientId){
    if(operationBusy||externalBusy||actionBlockedReason(client))return;
    licensed=true;
    const slot=snapshot.clients.find(s=>s.clientId===client);
    if(slot && !slot.profileExists){
      if(operationBusy||externalBusy)return;
      operationBusy=true;
      try{await onRelease?.();setupClient=client;setupDialog.showModal();}
      catch(e){const issue=operationIssue(e);error=issue.message+' '+issue.action;details=issue.details;}
      finally{operationBusy=false;}
    }
    else if(client==="01")void nativeLaunch();else void act(client,"Launch");
  }
  async function createAndOpen(){
    const client=setupClient;if(!client)return;setupDialog.close();compatibility=true;
    try{
      await act(client,"Create");
      if(current&&snapshot.clients.find(s=>s.clientId===client)?.settingsVerified)await act(client,"Launch");
    }finally{
      try{await onResume?.();}
      catch(e){const issue=operationIssue(e);error=issue.message;details=issue.details;}
    }
  }
  async function cancelSetup(){
    setupDialog.close();setupClient=null;
    try{await onResume?.();}
    catch(e){const issue=operationIssue(e);error=issue.message;details=issue.details;}
  }
  let current=true;
  let pending:Partial<Record<ClientId,boolean>>={};
  let closing:Partial<Record<ClientId,boolean>>={};
  $: snapshot=initialSnapshot;
  $: if(initialSnapshot)current=true;
  export async function closeClient(client:ClientId,target:OwnedProcess){await act(client,'Close',target);}
  export function actionBlockedReason(client:ClientId){return clientActionBlockedReason(snapshot,client,current,!!pending[client],!!closing[client]);}
  async function retryLaunch(client:ClientId){
    if(operationBusy||externalBusy||clientActionBlockedReason(snapshot,client,current,false,!!closing[client]))return;
    pending[client]=false;
    await openClient(client);
  }
  export async function activateClient(client:ClientId){
    if(operationBusy||externalBusy)return;
    const retained=client==='01'?snapshot.nativeProcesses[0]:snapshot.clients.find(slot=>slot.clientId===client)?.processes[0];
    if(current&&retained&&await play(client,retained))return;
    await act('02','Inspect');
    if(!current)return;
    const target=client==='01'?snapshot.nativeProcesses[0]:snapshot.clients.find(slot=>slot.clientId===client)?.processes[0];
    if(!target&&(client==='01'||snapshot.clients.find(slot=>slot.clientId===client)?.launchAllowed))pending[client]=false;
    if(actionBlockedReason(client))return;
    if(target)await play(client,target);else await openClient(client);
  }
  async function play(client:ClientId,target:OwnedProcess){
    if(!onPresent){await act(client,'Focus',target);return current;}
    if(operationBusy||externalBusy)return false;
    if(actionBlockedReason(client))return false;
    operationBusy=true;error='';details='';
    try{await onPresent(client,target);return true;}catch{return false;}
    finally{operationBusy=false;}
  }
  async function act(client:ClientId,operation:OwnedOperation,target:OwnedProcess|null=null){
    if(operationBusy||externalBusy)return;
    operationBusy=true;activeClient=client;activeOperation=operation;error="";notice="";details="";
    try{
      snapshot=await ownedProvider.execute(sessionDirectory,client,operation,licensed,compatibility,target);
      initialSnapshot=snapshot;
      current=true;
      if(operation==='Inspect')await onObserved?.(snapshot);
      if(operation==="Launch"){
        pending[client]=true;
        const slot=snapshot.clients.find(s=>s.clientId===client);
        if(snapshot.focusError)details=snapshot.focusError;
        notice=slot?.status==="RUNNING"&&snapshot.focusError?`Minecraft ${client} opened in the background. Click Focus to show it.`:slot?.status==="RUNNING"?`Minecraft ${client} window brought forward. Sign-in and gameplay need manual testing.`:
          `Launch requested for ${client}. Exit ${snapshot.launcherExitCode??"pending"}; no playable session is established. Refresh before any retry.`;
        if(onPresent){
          notice=`Starting Minecraft ${client}. Waiting for its game window…`;
          const presented=await observeWorkspaceLaunch(client,()=>ownedProvider.execute(sessionDirectory,client,'Inspect'),onPresent,observation=>{snapshot=observation;initialSnapshot=observation;current=true;});
          pending[client]=!presented;
          notice=presented?`Minecraft ${client} opened in the play area. Sign-in stays inside Minecraft.`:`Minecraft ${client} has not provided a responsive window yet. Check all clients before retrying.`;
        }
      }else if(operation==="Close"){closing[client]=true;notice="Normal close requested. Save/confirm inside Minecraft if needed; refresh to observe completion.";}
      else if(operation==="Create")notice=`Persistent profile ${client} created and settings verified.`;
      else if(operation==="Inspect")notice="Client status updated.";
      for(const slot of snapshot.clients){if(slot.processes.length>0)pending[slot.clientId]=false;else closing[slot.clientId]=false;}
      if(snapshot.nativeProcesses.length>0)pending["01"]=false;
      else closing['01']=false;
    }catch(e){const issue=operationIssue(e);const rejectedBeforeLaunch=operation==='Launch'&&issue.details.includes('APP_COMPARTMENT_FEATURE_UNAVAILABLE');if(!rejectedBeforeLaunch)current=false;error=issue.message+" "+issue.action;details=issue.details;if(operation==="Launch")pending[client]=!rejectedBeforeLaunch;}
    finally{operationBusy=false;activeClient="";activeOperation="";}
  }
  async function nativeLaunch(){
    if(operationBusy||externalBusy)return;
    operationBusy=true;activeClient="01";activeOperation="Launch";error="";notice="";details="";pending["01"]=true;
    try{
      const before=await ownedProvider.execute(sessionDirectory,"02","Inspect");
      if(!before.nativeProcesses.length)await nativeFeasibility.openOfficialMinecraft();
      notice='Starting Minecraft 01. Waiting for its game window…';
      const presented=await observeWorkspaceLaunch('01',()=>ownedProvider.execute(sessionDirectory,'02','Inspect'),onPresent,observation=>{snapshot=observation;initialSnapshot=observation;current=true;});
      pending['01']=!presented;
      notice=presented?'Minecraft 01 opened in the play area. Sign-in stays inside Minecraft.':'Minecraft 01 has not provided a responsive window yet. Check all clients before retrying.';
    }
    catch(e){const issue=operationIssue(e);error=issue.message+" "+issue.action;details=issue.details;current=false;}
    finally{operationBusy=false;activeClient="";activeOperation="";}
  }
  $: disabled=operationBusy||externalBusy;
  $: native=snapshot.nativeProcesses[0]??null;
  $: tabStates={
    '02':'Checking','03':'Checking','04':'Checking',
    '01':activeClient==='01'&&activeOperation==='Launch'?'Opening':!current?'Check needed':native?(native.responding?'Running':'Not responding'):pending['01']?'Check needed':'Not opened',
    ...Object.fromEntries(snapshot.clients.map(slot=>[slot.clientId,ownedClientStatus(slot,current,!!pending[slot.clientId],activeClient===slot.clientId?activeOperation as OwnedOperation:null,!!closing[slot.clientId])]))
  };
</script>

<div class="nw-session-tools"><p>Switch tabs to keep each session running. Closing a game is a separate action.</p><button class="cm-button cm-secondary" disabled={disabled} on:click={()=>act('02','Inspect')}>{operationBusy?'Working…':'Refresh sessions'}</button></div>
{#if error}<div class="nw-operation-error" role="alert">{error}<details><summary>Details</summary><pre>{details}</pre></details></div>{/if}
<dialog class="um-profile-dialog" bind:this={setupDialog} aria-labelledby="um-profile-title" on:cancel|preventDefault={cancelSetup}>
  <h2 id="um-profile-title">Prepare Minecraft {setupClient}</h2>
  <p>We will create its persistent game profile, then open Minecraft. Existing profiles and worlds will be preserved.</p>
  <p>The Minecraft compatibility profile reduces process and COM containment so the game can work with Windows services. It does not prove account isolation.</p>
  <div><button class="cm-button cm-secondary" on:click={cancelSetup}>Cancel</button><button class="cm-button" on:click={createAndOpen}>Prepare and open</button></div>
</dialog>
