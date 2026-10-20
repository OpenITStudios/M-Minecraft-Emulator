<script lang="ts">
  import {ownedProvider,type OwnedSnapshot,type OwnedOperation,type ClientId,type OwnedProcess} from "./app/bridge/ownedProviderApi.js";
  import {nativeFeasibility} from "./app/bridge/nativeFeasibilityApi.js";
  export let sessionDirectory:string;
  export let initialSnapshot:OwnedSnapshot;
  export let operationBusy=false;
  export let externalBusy=false;
  let snapshot=initialSnapshot;
  let error="",notice="";
  let licensed=false,compatibility=false;
  let current=true;
  let pending:Partial<Record<ClientId,boolean>>={};
  async function act(client:ClientId,operation:OwnedOperation,target:OwnedProcess|null=null){
    if(operationBusy||externalBusy)return;
    operationBusy=true;error="";notice="";
    try{
      snapshot=await ownedProvider.execute(sessionDirectory,client,operation,licensed,compatibility,target);
      initialSnapshot=snapshot;
      current=true;
      if(operation==="Launch"){
        pending[client]=true;
        const slot=snapshot.clients.find(s=>s.clientId===client);
        notice=slot?.status==="RUNNING"?`Minecraft ${client} window observed. Sign-in and gameplay need manual testing.`:
          `Launch requested for ${client}. Exit ${snapshot.launcherExitCode??"pending"}; no playable session is established. Refresh before any retry.`;
      }else if(operation==="Close")notice="Normal close requested. Save/confirm inside Minecraft if needed; refresh to observe completion.";
      else if(operation==="Create")notice=`Persistent profile ${client} created and settings verified.`;
      else if(operation==="Inspect")notice="Fresh engine/profile/process inspection completed.";
      for(const slot of snapshot.clients){if(slot.processes.length>0)pending[slot.clientId]=false;}
      if(snapshot.nativeProcesses.length>0)pending["01"]=false;
    }catch(e){current=false;error=String(e);if(operation==="Launch")pending[client]=true;}
    finally{operationBusy=false;}
  }
  async function nativeLaunch(){
    if(operationBusy||externalBusy||!licensed)return;
    operationBusy=true;error="";notice="";pending["01"]=true;
    try{await nativeFeasibility.openOfficialMinecraft();notice="Native Education launch requested. Refresh to observe its process; login remains manual.";current=false;}
    catch(e){error=String(e);current=false;}
    finally{operationBusy=false;}
  }
  $: disabled=operationBusy||externalBusy;
  $: native=snapshot.nativeProcesses[0]??null;
</script>

<section class="cm-virtual-overview" aria-label="Owned engine test mode">
  <div><h2>Local engine test · Minecraft 01–04</h2><p>Engine checks run on demand. A detected window does not establish login, account isolation or multiplayer.</p></div>
  <button class="cm-button cm-secondary" disabled={disabled} on:click={()=>act("02","Inspect")}>{operationBusy?"Working…":"Refresh all clients"}</button>
</section>
<div class="cm-setting-group">
  <label><input type="checkbox" bind:checked={licensed} disabled={disabled}/> I have a separate licensed Education account for each session I launch.</label>
  <label><input type="checkbox" bind:checked={compatibility} disabled={disabled}/> Use the Minecraft compatibility profile with reduced process/COM containment for new virtuals.</label>
</div>
{#if error}<p class="cm-warning" role="alert">{error}</p>{/if}
{#if notice}<p role="status">{notice}</p>{/if}
{#if !current}<p role="status">Previous observation; refresh before another operation.</p>{/if}
<section class="cm-list" aria-label="Owned mode sessions">
  <div class="cm-instance"><div class="cm-row"><div class="cm-row-copy"><h2>Native · Minecraft 01</h2><p>{native?"Attributed native process observed":"No native process observed"}</p></div>
    <div class="cm-row-actions">
      <button class="cm-button" disabled={disabled||!current||!licensed||!!native||!!pending["01"]} on:click={nativeLaunch}>Open 01</button>
      {#if pending["01"] && current && !native}
        <button class="cm-button cm-secondary" disabled={disabled||!licensed} on:click={nativeLaunch}>Retry after empty native check</button>
      {/if}
      <button class="cm-button cm-secondary" disabled={disabled||!current||!native||native.windowHandle==="0x0"} on:click={()=>act("01","Focus",native)}>Focus 01</button>
      <button class="cm-button cm-secondary" disabled={disabled||!current||!native||native.windowHandle==="0x0"} on:click={()=>act("01","PlaceLeft",native)}>Left</button>
      <button class="cm-button cm-secondary" disabled={disabled||!current||!native||native.windowHandle==="0x0"} on:click={()=>act("01","PlaceRight",native)}>Right</button>
      <button class="cm-button cm-secondary" disabled={disabled||!current||!native||native.windowHandle==="0x0"} on:click={()=>act("01","Close",native)}>Close normally</button>
    </div></div></div>
  {#each snapshot.clients as slot (slot.clientId)}
    {@const target=slot.processes[0]??null}
    <div class="cm-instance"><div class="cm-row"><div class="cm-row-copy"><h2>Virtual · Minecraft {slot.clientId}</h2><p>{slot.status} · {slot.box}</p></div>
      <div class="cm-row-actions">
        <button class="cm-button cm-secondary" disabled={disabled||!current||slot.profileExists||!compatibility} on:click={()=>act(slot.clientId,"Create")}>Create {slot.clientId}</button>
        <button class="cm-button" disabled={disabled||!current||!licensed||!slot.settingsVerified||slot.boxedProcessCount!==0||!!pending[slot.clientId]} on:click={()=>act(slot.clientId,"Launch")}>Open {slot.clientId}</button>
        {#if pending[slot.clientId] && current && slot.boxedProcessCount===0}
          <button class="cm-button cm-secondary" disabled={disabled||!licensed} on:click={()=>act(slot.clientId,"Launch")}>Retry after empty check · wait 30 seconds</button>
        {/if}
        <button class="cm-button cm-secondary" disabled={disabled||!current||!target||target.windowHandle==="0x0"} on:click={()=>act(slot.clientId,"Focus",target)}>Focus</button>
        <button class="cm-button cm-secondary" disabled={disabled||!current||!target||target.windowHandle==="0x0"} on:click={()=>act(slot.clientId,"PlaceLeft",target)}>Left</button>
        <button class="cm-button cm-secondary" disabled={disabled||!current||!target||target.windowHandle==="0x0"} on:click={()=>act(slot.clientId,"PlaceRight",target)}>Right</button>
        <button class="cm-button cm-secondary" disabled={disabled||!current||!target||target.windowHandle==="0x0"} on:click={()=>act(slot.clientId,"Close",target)}>Close normally</button>
      </div></div></div>
  {/each}
</section>
