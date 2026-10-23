<script lang="ts">
  import {onMount,tick} from 'svelte';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import type {ClientId,OwnedProcess,OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {operationIssue} from './app/bridge/managerConfiguration.js';
  import {listen} from '@tauri-apps/api/event';
  export let sessionDirectory='';
  export let releaseOnly=false;
  export let activeClient:ClientId|null=null;
  export let busy=false;
  export let tabStates:Record<ClientId,string>={'01':'Checking','02':'Checking','03':'Checking','04':'Checking'};
  export let onClose:(client:ClientId,target:OwnedProcess)=>Promise<void>=async()=>{};
  export let onSelect:(client:ClientId)=>Promise<void>=async()=>{};
  export let onRecovered:()=>void=()=>{};
  export let selectionBlocked:(client:ClientId)=>string|null=()=>null;
  const clientIds:ClientId[]=['01','02','03','04'];
  function tabKey(event:KeyboardEvent){
    const tabs=Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('[role=tab]')).filter(tab=>!tab.disabled);
    if(!tabs.length)return;
    const index=tabs.indexOf(document.activeElement as HTMLButtonElement);
    const next=event.key==='ArrowRight'?(index+1)%tabs.length:event.key==='ArrowLeft'?(index+tabs.length-1)%tabs.length:event.key==='Home'?0:event.key==='End'?tabs.length-1:-1;
    if(next>=0){event.preventDefault();tabs[next]?.focus();}
  }
  let viewport:HTMLDivElement,target:OwnedProcess|null=null;
  let failedPresentation:{client:ClientId;target:OwnedProcess}|null=null;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let updating=false,updatePending=false;
  let suspended=false;
  export let working=false;
  $: disabled=busy||working;
  $: if(!busy&&!working&&!suspended&&updatePending&&activeClient&&!updating)void update();
  export async function restore(){
    await workspaceSurface.release();activeClient=null;target=null;failedPresentation=null;issue=null;suspended=false;
  }
  export async function suspend(){
    const previous=suspended;suspended=true;updatePending=false;
    try{const retained=await workspaceSurface.suspend();if(!retained){activeClient=null;target=null;}}
    catch(error){suspended=previous;throw error;}
  }
  export async function resume(){if(suspended&&activeClient&&target)await present(activeClient,target);}
  export async function reconcile(observation:OwnedSnapshot){
    if(!activeClient||!target)return;
    const processes=activeClient==='01'?observation.nativeProcesses:observation.clients.find(slot=>slot.clientId===activeClient)?.processes??[];
    const previous=target;
    if(processes.some(process=>process.pid===previous.pid&&process.createdFileTime===previous.createdFileTime&&process.sessionId===previous.sessionId&&process.windowHandle===previous.windowHandle))return;
    await workspaceSurface.release(previous);
    activeClient=null;target=null;issue=null;suspended=false;updatePending=false;
  }
  export async function present(client:ClientId,process:OwnedProcess){
    issue=null;
    try{
      await tick();
      await workspaceSurface.present(sessionDirectory,client,process,viewport.getBoundingClientRect());
      activeClient=client;target=process;failedPresentation=null;suspended=false;
      issue=null;onRecovered();
    }catch(error){failedPresentation={client,target:process};issue=operationIssue(error);throw error;}
  }
  async function update(){
    if(suspended)return;
    updatePending=true;if(updating||busy||working||!activeClient)return;
    updating=true;
    try{while(updatePending&&activeClient){updatePending=false;await workspaceSurface.update(viewport.getBoundingClientRect());if(!failedPresentation)issue=null;}}
    catch(error){issue??=operationIssue(error);updatePending=false;}
    finally{updating=false;}
  }
  async function returnToWindows(){
    if(disabled)return;working=true;issue=null;
    try{await restore();}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  async function retryDisplay(){
    const retry=failedPresentation?.client??activeClient;
    if(disabled||!retry)return;
    await onSelect(retry);
  }
  async function closeClient(){
    if(disabled||!activeClient||!target)return;
    const client=activeClient,process=target;working=true;issue=null;
    try{if(releaseOnly){await restore();}else{await workspaceSurface.release(process);activeClient=null;target=null;working=false;await tick();await onClose(client,process);}}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  onMount(()=>{
    let disposed=false,unlisten:(()=>void)|undefined;
    void listen<string>('workspace-surface-error',event=>{issue??=operationIssue(event.payload);updatePending=false;}).then(stop=>{if(disposed)stop();else unlisten=stop;}).catch(error=>{if(!disposed)issue=operationIssue(`Workspace recovery notifications unavailable: ${String(error)}`);});
    const observer=new ResizeObserver(()=>{void update();});observer.observe(viewport,{box:'device-pixel-content-box'});
    return ()=>{disposed=true;unlisten?.();observer.disconnect();void workspaceSurface.release().catch(()=>{});};
  });
</script>
<section class="nw-stage" id="minecraft-play-area" aria-label="Active Minecraft session">
  <header><div><p class="um-eyebrow">PLAY AREA</p><h2>{activeClient?`Minecraft ${activeClient}`:'Your Minecraft workspace'}</h2></div>
    {#if activeClient}<div class="nw-actions"><button class="cm-button cm-secondary" {disabled} on:click={closeClient}>{releaseOnly?"Return to Windows":"Close game"}</button></div>{/if}
  </header>
  <div class="nw-tabs" role="tablist" tabindex="-1" aria-label="Minecraft sessions" on:keydown={tabKey}>
    {#each clientIds as client}<button role="tab" aria-selected={activeClient===client} aria-controls="minecraft-play-area" title={selectionBlocked(client)??''} disabled={disabled||!!selectionBlocked(client)} on:click={()=>onSelect(client)}><span>Minecraft {client}</span><small>{tabStates[client]}</small></button>{/each}
  </div>
  <div class="nw-viewport"><div class="nw-native-slot" bind:this={viewport}>
    {#if !activeClient}<div><span class="nw-mark">M</span><h3>Choose a client to play</h3><p>Select one tab to open its session. Other tabs keep running.</p></div>{/if}
  </div></div>
  <div class="nw-feedback" aria-live="polite">
    {#if issue}<div role="alert"><strong>{failedPresentation?'Could not show Minecraft '+failedPresentation.client:issue.message}</strong> <span>{activeClient?'Minecraft '+activeClient+' remains selected.':'Choose a tab to try again.'}</span> {#if failedPresentation||(activeClient&&target)}<button class="cm-button cm-secondary" {disabled} on:click={retryDisplay}>Try again</button>{/if}<details><summary>Details</summary><pre>{issue.details}</pre><button class="cm-button cm-secondary" {disabled} on:click={returnToWindows}>Release all windows to Windows</button></details></div>
    {:else}<p>Only the selected tab is shown. Other sessions stay running. Click inside the game to control it.</p>{/if}
  </div>
</section>
