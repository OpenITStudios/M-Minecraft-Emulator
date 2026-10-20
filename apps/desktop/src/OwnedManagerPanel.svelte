<script lang="ts">
  import {ownedProvider,type OwnedSnapshot,type OwnedOperation,type ClientId,type OwnedProcess} from "./app/bridge/ownedProviderApi.js";
  import {ownedClientStatus} from "./app/presentation/unifiedManager.js";
  import {operationIssue} from "./app/bridge/managerConfiguration.js";
  import {nativeFeasibility} from "./app/bridge/nativeFeasibilityApi.js";
  import {observeWorkspaceLaunch} from './app/presentation/launchWorkspace.js';
  export let sessionDirectory:string;
  export let initialSnapshot:OwnedSnapshot;
  export let operationBusy=false;
  export let externalBusy=false;
  export let selectedClient:ClientId|null=null;
  export let onPresent:((client:ClientId,target:OwnedProcess)=>Promise<void>)|null=null;
  export let onRelease:(()=>Promise<void>)|null=null;
  export let onResume:(()=>Promise<void>)|null=null;
  let snapshot=initialSnapshot;
  let error="",notice="",details="",activeClient="",activeOperation="";
  let licensed=false,compatibility=false;
  let setupDialog:HTMLDialogElement;
  let setupClient:ClientId|null=null;
  async function openClient(client:ClientId){
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
    await act(client,"Create");
    if(current&&snapshot.clients.find(s=>s.clientId===client)?.settingsVerified)await act(client,"Launch");
    if(!current)await onResume?.();
  }
  let current=true;
  let pending:Partial<Record<ClientId,boolean>>={};
  let closing:Partial<Record<ClientId,boolean>>={};
  $: snapshot=initialSnapshot;
  $: if(initialSnapshot)current=true;
  export async function closeClient(client:ClientId,target:OwnedProcess){await act(client,'Close',target);}
  export async function activateClient(client:ClientId){
    const target=client==='01'?snapshot.nativeProcesses[0]:snapshot.clients.find(slot=>slot.clientId===client)?.processes[0];
    if(target)await play(client,target);else await openClient(client);
  }
  async function play(client:ClientId,target:OwnedProcess){
    if(!onPresent){await act(client,'Focus',target);return;}
    if(operationBusy||externalBusy)return;
    operationBusy=true;error='';details='';
    try{await onPresent(client,target);}catch(e){const issue=operationIssue(e);error=issue.message+' '+issue.action;details=issue.details;}
    finally{operationBusy=false;}
  }
  async function act(client:ClientId,operation:OwnedOperation,target:OwnedProcess|null=null){
    if(operationBusy||externalBusy)return;
    operationBusy=true;activeClient=client;activeOperation=operation;error="";notice="";details="";
    try{
      snapshot=await ownedProvider.execute(sessionDirectory,client,operation,licensed,compatibility,target);
      initialSnapshot=snapshot;
      current=true;
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
    }catch(e){current=false;const issue=operationIssue(e);error=issue.message+" "+issue.action;details=issue.details;if(operation==="Launch")pending[client]=true;}
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
</script>

<div class="um-toolbar"><div><strong>Owned engine connected</strong><p>Statuses show process identity. Sign-in and gameplay are tested manually.</p></div><button class="cm-button cm-secondary" disabled={disabled} on:click={()=>act("02","Inspect")}>{operationBusy?"Working…":"Check all clients"}</button></div>
<p class="um-license">Choosing Open confirms you will sign in with a separate licensed Education account for that client. Sign-in stays inside Minecraft.</p>
{#if error}<div class="um-alert" role="alert">{error}<details><summary>Diagnostics</summary><pre>{details}</pre></details></div>{/if}
{#if notice}<p class="um-notice" role="status">{notice}</p>{/if}
{#if !current}<p class="um-notice" role="status">Status needs a fresh check before another operation.</p>{/if}
<section class="um-grid" aria-label="Minecraft clients">
  <article class="um-card" hidden={selectedClient==='01'}><div class="um-card-top"><span class="um-number">01</span><span class="um-eyebrow">NATIVE</span></div><h2>Minecraft 01</h2><span class="um-status" class:running={!!native}>{activeClient==='01'?'Starting':!current?'Needs attention':closing['01']&&native?'Closing':native?'Running':pending['01']?'Needs attention':'Ready'}</span><p>Windows Desktop installation</p>
    <button class="cm-button um-primary" disabled={disabled||!current||(!native&&!!pending['01'])||(!!native&&native.windowHandle==='0x0')} on:click={()=>native?play('01',native):openClient('01')}>{native?(onPresent?'Play':'Focus'):'Open'}</button>
    {#if pending['01'] && current && !native}<button class="cm-button cm-secondary" disabled={disabled} on:click={()=>openClient("01")}>Retry after empty native check</button>{/if}
    <details class="um-actions"><summary>Window actions</summary><div><button disabled={disabled||!current||!native||native.windowHandle==='0x0'} on:click={()=>act('01','PlaceLeft',native)}>Place left</button><button disabled={disabled||!current||!native||native.windowHandle==='0x0'} on:click={()=>act('01','PlaceRight',native)}>Place right</button><button disabled={disabled||!current||!native||native.windowHandle==='0x0'} on:click={()=>act('01','Close',native)}>Close normally</button></div></details>
  </article>
  {#each snapshot.clients as slot (slot.clientId)}
    {@const target=slot.processes[0]??null}
    <article class="um-card" hidden={selectedClient===slot.clientId}><div class="um-card-top"><span class="um-number">{slot.clientId}</span><span class="um-eyebrow">OWNED</span></div><h2>Minecraft {slot.clientId}</h2><span class="um-status" class:running={slot.status==='RUNNING'}>{ownedClientStatus(slot,current,!!pending[slot.clientId],activeClient===slot.clientId?activeOperation as OwnedOperation:null,!!closing[slot.clientId])}</span><p>{slot.box} · Persistent profile</p>
      {#if !slot.profileExists}
        <button class="cm-button um-primary" disabled={disabled||!current} on:click={()=>openClient(slot.clientId)}>Open</button>
      {:else if target}
        <button class="cm-button um-primary" disabled={disabled||!current||target.windowHandle==='0x0'} on:click={()=>play(slot.clientId,target)}>{onPresent?'Play':'Focus'}</button>
      {:else}
        <button class="cm-button um-primary" disabled={disabled||!current||!slot.settingsVerified||!slot.launchAllowed||slot.boxedProcessCount!==0||!!pending[slot.clientId]} on:click={()=>openClient(slot.clientId)}>Open</button>
      {/if}
      {#if pending[slot.clientId] && current && slot.boxedProcessCount===0}<button class="cm-button cm-secondary" disabled={disabled||!slot.launchAllowed} on:click={()=>openClient(slot.clientId)}>Retry after empty check · wait 30 seconds</button>{/if}
      <details class="um-actions"><summary>Window actions</summary><div><button disabled={disabled||!current||!target||target.windowHandle==='0x0'} on:click={()=>act(slot.clientId,'PlaceLeft',target)}>Place left</button><button disabled={disabled||!current||!target||target.windowHandle==='0x0'} on:click={()=>act(slot.clientId,'PlaceRight',target)}>Place right</button><button disabled={disabled||!current||!target||target.windowHandle==='0x0'} on:click={()=>act(slot.clientId,'Close',target)}>Close normally</button></div></details>
    </article>
  {/each}
</section>
<dialog class="um-profile-dialog" bind:this={setupDialog} aria-labelledby="um-profile-title">
  <h2 id="um-profile-title">Prepare Minecraft {setupClient}</h2>
  <p>We will create its persistent game profile, then open Minecraft. Existing profiles and worlds will be preserved.</p>
  <p>The Minecraft compatibility profile reduces process and COM containment so the game can work with Windows services. It does not prove account isolation.</p>
  <div><button class="cm-button cm-secondary" on:click={async()=>{setupDialog.close();try{await onResume?.();}catch(e){const issue=operationIssue(e);error=issue.message;details=issue.details;}}}>Cancel</button><button class="cm-button" on:click={createAndOpen}>Prepare and open</button></div>
</dialog>
