<script lang="ts">
  import {onMount,tick} from 'svelte';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import type {ClientId,OwnedProcess} from './app/bridge/ownedProviderApi.js';
  import {operationIssue} from './app/bridge/managerConfiguration.js';
  export let sessionDirectory='';
  export let activeClient:ClientId|null=null;
  export let busy=false;
  export let onClose:(client:ClientId,target:OwnedProcess)=>Promise<void>=async()=>{};
  let viewport:HTMLDivElement,target:OwnedProcess|null=null;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let updating=false,updatePending=false;
  export let working=false;
  $: disabled=busy||working;
  $: if(!busy&&updatePending&&activeClient&&!updating)void update();
  export async function restore(){
    await workspaceSurface.release();activeClient=null;target=null;
  }
  export async function present(client:ClientId,process:OwnedProcess){
    issue=null;
    try{
      await tick();
      await workspaceSurface.present(sessionDirectory,client,process,viewport.getBoundingClientRect());
      activeClient=client;target=process;
    }catch(error){issue=operationIssue(error);if(/recovery not confirmed|Previous surface could not resume|Surface helper ended/.test(issue.details)){activeClient=null;target=null;}throw error;}
  }
  async function update(){
    updatePending=true;if(updating||busy||!activeClient)return;
    updating=true;
    try{while(updatePending&&activeClient){updatePending=false;await workspaceSurface.update(viewport.getBoundingClientRect());}}
    catch(error){issue=operationIssue(error);activeClient=null;target=null;}
    finally{updating=false;}
  }
  async function returnToWindows(){
    if(disabled)return;working=true;issue=null;
    try{await restore();}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  async function closeClient(){
    if(disabled||!activeClient||!target)return;
    const client=activeClient,process=target;working=true;issue=null;
    try{await restore();working=false;await tick();await onClose(client,process);}catch(error){issue=operationIssue(error);}finally{working=false;}
  }
  onMount(()=>{
    const observer=new ResizeObserver(()=>{void update();});observer.observe(viewport,{box:'device-pixel-content-box'});
    return ()=>{observer.disconnect();void workspaceSurface.release().catch(()=>{});};
  });
</script>
<section class="nw-stage" aria-label="Active Minecraft session">
  <header><div><p class="um-eyebrow">PLAY AREA</p><h2>{activeClient?`Minecraft ${activeClient}`:'Your Minecraft workspace'}</h2></div>
    {#if activeClient||issue}<div class="nw-actions"><button class="cm-button cm-secondary" {disabled} on:click={returnToWindows}>Restore window</button>{#if activeClient}<button class="cm-button cm-secondary" {disabled} on:click={closeClient}>Close normally</button>{/if}</div>{/if}
  </header>
  {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
  <div class="nw-viewport"><div class="nw-native-slot" bind:this={viewport}>
    {#if !activeClient}<div><span class="nw-mark">M</span><h3>Choose a client to play</h3><p>Open a session or select Play. Sign in inside Minecraft.</p></div>{/if}
  </div></div>
  <p class="nw-footnote">{activeClient?'Click inside Minecraft to play · Accounts stay inside the game':'Windows workspace · One native and three private sessions'}</p>
</section>
