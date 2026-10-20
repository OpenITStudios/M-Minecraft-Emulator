<script lang="ts">
  import {onMount,tick} from 'svelte';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import type {ClientId,OwnedProcess,OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {operationIssue} from './app/bridge/managerConfiguration.js';
  import {listen} from '@tauri-apps/api/event';
  export let sessionDirectory='';
  export let activeClient:ClientId|null=null;
  export let busy=false;
  export let onClose:(client:ClientId,target:OwnedProcess)=>Promise<void>=async()=>{};
  export let onSelect:(client:ClientId)=>Promise<void>=async()=>{};
  const clientIds:ClientId[]=['01','02','03','04'];
  function tabKey(event:KeyboardEvent){
    const tabs=Array.from((event.currentTarget as HTMLElement).querySelectorAll<HTMLButtonElement>('[role=tab]'));
    const index=tabs.indexOf(document.activeElement as HTMLButtonElement);
    const next=event.key==='ArrowRight'?(index+1)%4:event.key==='ArrowLeft'?(index+3)%4:event.key==='Home'?0:event.key==='End'?3:-1;
    if(next>=0){event.preventDefault();tabs[next]?.focus();}
  }
  let viewport:HTMLDivElement,target:OwnedProcess|null=null;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let updating=false,updatePending=false;
  let updateFailed=false,suspended=false;
  export let working=false;
  $: disabled=busy||working;
  $: if(!busy&&!working&&!suspended&&updatePending&&activeClient&&!updating)void update();
  export async function restore(){
    await workspaceSurface.release();activeClient=null;target=null;updateFailed=false;issue=null;suspended=false;
  }
  export async function suspend(){
    const previous=suspended;suspended=true;updatePending=false;
    try{const retained=await workspaceSurface.suspend();if(!retained){activeClient=null;target=null;updateFailed=false;}}
    catch(error){suspended=previous;throw error;}
  }
  export async function resume(){if(activeClient&&target)await present(activeClient,target);}
  export async function reconcile(observation:OwnedSnapshot){
    if(!activeClient||!target)return;
    const processes=activeClient==='01'?observation.nativeProcesses:observation.clients.find(slot=>slot.clientId===activeClient)?.processes??[];
    const previous=target;
    if(processes.some(process=>process.pid===previous.pid&&process.createdFileTime===previous.createdFileTime&&process.sessionId===previous.sessionId&&process.windowHandle===previous.windowHandle))return;
    await workspaceSurface.release(previous);
    activeClient=null;target=null;issue=null;updateFailed=false;suspended=false;updatePending=false;
  }
  export async function present(client:ClientId,process:OwnedProcess){
    issue=null;
    try{
      await tick();
      await workspaceSurface.present(sessionDirectory,client,process,viewport.getBoundingClientRect());
      activeClient=client;target=process;updateFailed=false;suspended=false;
    }catch(error){issue=operationIssue(error);throw error;}
  }
  async function update(){
    if(updateFailed||suspended)return;
    updatePending=true;if(updating||busy||working||!activeClient)return;
    updating=true;
    try{while(updatePending&&activeClient){updatePending=false;await workspaceSurface.update(viewport.getBoundingClientRect());}}
    catch(error){issue=operationIssue(error);updateFailed=true;updatePending=false;}
    finally{updating=false;}
  }
  async function returnToWindows(){
    if(disabled)return;working=true;issue=null;
    try{await restore();}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  async function closeClient(){
    if(disabled||!activeClient||!target)return;
    const client=activeClient,process=target;working=true;issue=null;
    try{await workspaceSurface.release(process);activeClient=null;target=null;updateFailed=false;working=false;await tick();await onClose(client,process);}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  onMount(()=>{
    let disposed=false,unlisten:(()=>void)|undefined;
    void listen<string>('workspace-surface-error',event=>{issue=operationIssue(event.payload);updateFailed=true;updatePending=false;}).then(stop=>{if(disposed)stop();else unlisten=stop;}).catch(error=>{if(!disposed)issue=operationIssue(`Workspace recovery notifications unavailable: ${String(error)}`);});
    const observer=new ResizeObserver(()=>{void update();});observer.observe(viewport,{box:'device-pixel-content-box'});
    return ()=>{disposed=true;unlisten?.();observer.disconnect();void workspaceSurface.release().catch(()=>{});};
  });
</script>
<section class="nw-stage" id="minecraft-play-area" aria-label="Active Minecraft session">
  <header><div><p class="um-eyebrow">PLAY AREA</p><h2>{activeClient?`Minecraft ${activeClient}`:'Your Minecraft workspace'}</h2></div>
    {#if activeClient||issue}<div class="nw-actions"><button class="cm-button cm-secondary" {disabled} on:click={returnToWindows}>Restore window</button>{#if activeClient}<button class="cm-button cm-secondary" {disabled} on:click={closeClient}>Close normally</button>{/if}</div>{/if}
  </header>
  <div class="nw-tabs" role="tablist" tabindex="-1" aria-label="Minecraft sessions" on:keydown={tabKey}>
    {#each clientIds as client}<button role="tab" aria-selected={activeClient===client} aria-controls="minecraft-play-area" {disabled} on:click={()=>onSelect(client)}>Minecraft {client}</button>{/each}
  </div>
  {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
  <div class="nw-viewport"><div class="nw-native-slot" bind:this={viewport}>
    {#if !activeClient}<div><span class="nw-mark">M</span><h3>Choose a client to play</h3><p>Open a session or select Play. Sign in inside Minecraft.</p></div>{/if}
  </div></div>
  <p class="nw-footnote">{activeClient?'Click inside Minecraft to play · Accounts stay inside the game':'Windows workspace · One native and three private sessions'}</p>
</section>
