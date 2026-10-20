<script lang="ts">
  import {onMount,tick} from 'svelte';
  import OwnedManagerPanel from './OwnedManagerPanel.svelte';
  import NativeWorkspace from './NativeWorkspace.svelte';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import type {ClientId} from './app/bridge/ownedProviderApi.js';
  import type {OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {managerConfiguration,operationIssue,type ManagerConfiguration} from './app/bridge/managerConfiguration.js';
  let config:ManagerConfiguration|null=null,snapshot:OwnedSnapshot|null=null;
  let page:'clients'|'settings'='clients',busy=true,checked=false;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let workspace:NativeWorkspace,panel:OwnedManagerPanel,selectedClient:ClientId|null=null,workspaceBusy=false;
  async function navigate(next:'clients'|'settings'){
    if(busy||workspaceBusy)return;
    busy=true;
    try{if(next==='settings'){if(workspace)await workspace.suspend();else await workspaceSurface.suspend();page=next;}else{page=next;await tick();if(workspace)await workspace.resume();}}
    catch(error){issue=operationIssue(error);}finally{busy=false;}
  }
  async function prepare(){
    busy=true;checked=false;issue=null;
    try{config=await managerConfiguration.load();snapshot=config.observation;checked=!!snapshot;if(config.engineError)issue=operationIssue(config.engineError);await tick();if(snapshot&&workspace){try{await workspace.reconcile(snapshot);}catch(error){issue=operationIssue(error);}}}
    catch(e){snapshot=null;issue=operationIssue(e);}finally{busy=false;}
  }
  onMount(()=>{void (async()=>{
    await prepare();await tick();if(!snapshot||!workspace)return;
    busy=true;
    try{
      const running=snapshot.clients.filter(slot=>slot.status==='RUNNING'&&slot.processes[0]?.responding).map(slot=>({client:slot.clientId as ClientId,target:slot.processes[0]}));
      const native=snapshot.nativeProcesses[0];if(native&&native.responding&&native.windowHandle!=='0x0')running.push({client:'01',target:native});
      // A client exiting during admission must not prevent adopting the other sessions.
      for(const session of running){
        try{await workspace.present(session.client,session.target);}
        catch{/* The workspace owns presentation errors and recovery controls. */}
      }
    }catch(error){issue=operationIssue(error);}finally{busy=false;}
  })();});
</script>

<svelte:head><title>Minecraft Instance Manager</title></svelte:head>
<div class="um-app">
  <aside class="um-sidebar"><div class="um-brand"><span class="um-logo">M</span><div>Minecraft<span>Instance Manager</span></div></div>
    <nav aria-label="Main navigation"><button class:active={page==='clients'} disabled={busy||workspaceBusy} on:click={()=>navigate('clients')}>Clients <span>04</span></button><button class:active={page==='settings'} disabled={busy||workspaceBusy} on:click={()=>navigate('settings')}>Settings</button></nav>
    <p class="um-sidebar-foot">Windows developer test<br/>One native · Three owned</p>
  </aside>
  <main class="um-main">
    <header class="um-header"><div><p class="um-eyebrow">YOUR WORKSPACE</p><h1>{page==='clients'?'Minecraft clients':'Settings'}</h1><p>{page==='clients'?'Four sessions. One place to launch and manage.':'Minecraft and the engine are detected automatically.'}</p></div><span class="um-engine" class:ready={checked}>Engine · {busy?'Checking':checked?'Connected':'Needs attention'}</span></header>
    {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
    <div class="um-client-view" hidden={page!=='clients'}>
      {#if config && snapshot}<div class="nw-layout"><NativeWorkspace bind:this={workspace} bind:working={workspaceBusy} sessionDirectory={config.settings.sessionDirectory} bind:activeClient={selectedClient} {busy} onRecovered={()=>issue=null} selectionBlocked={client=>panel?.actionBlockedReason(client)??null} onClose={(client,target)=>panel.closeClient(client,target)} onSelect={client=>panel.activateClient(client)}/><aside class="nw-sessions" aria-label="Other Minecraft sessions"><OwnedManagerPanel bind:this={panel} sessionDirectory={config.settings.sessionDirectory} bind:initialSnapshot={snapshot} bind:operationBusy={busy} {selectedClient} externalBusy={workspaceBusy} onPresent={(client,target)=>workspace.present(client,target)} onRelease={()=>workspace.suspend()} onResume={()=>workspace.resume()} onObserved={observation=>workspace.reconcile(observation)}/></aside></div>
      {:else}
        <section class="um-setup"><h2>{busy?'Checking your engine…':'Engine needs attention'}</h2><p>Automatic verification could not connect to the engine. Your profiles and game data are preserved.</p><button class="cm-button" disabled={busy} on:click={prepare}>Check again</button><button class="cm-button cm-secondary" disabled={busy} on:click={()=>page='settings'}>View status</button></section>
        <section class="um-grid" aria-label="Minecraft clients">{#each config?.catalog.targets??[] as target}<article class="um-card"><p class="um-eyebrow">{target.kind==='NATIVE'?'NATIVE':'OWNED'}</p><h2>Minecraft {target.id}</h2><span class="um-status">Waiting for engine</span><p>{target.kind==='NATIVE'?'Native session':'Private persistent session'}</p></article>{/each}</section>
      {/if}
    </div>
    {#if page==='settings'}
      <section class="um-settings"><h2>Automatic setup</h2><p>No folders or executable files to choose. We check the installed game and engine before use.</p>
        <div class="um-setting"><div><h3>Minecraft Education</h3><p>{busy?'Checking installation…':config?.targetError?'Installation needs attention. Repair Minecraft Education, then check again.':config?.detectedMinecraft?'Installed · detected automatically':'Not detected. Install Minecraft Education Desktop, then check again.'}</p></div></div>
        <div class="um-setting"><div><h3>Engine</h3><p>{busy?'Checking engine…':checked?'Connected · verified for this session':'Unavailable. Check Diagnostics for the reason.'}</p></div></div>
        <button class="cm-button" disabled={busy} on:click={prepare}>{busy?'Checking…':'Check again'}</button>
      </section>
      <section class="um-settings"><details><summary>Diagnostics</summary><p>Windows developer test. Independent installation and production signing have separate acceptance requirements.</p><h3>Minecraft location</h3><pre>{config?.settings.minecraftExecutable||config?.detectedMinecraft||'Not detected'}</pre><h3>Engine location</h3><pre>{config?.settings.sessionDirectory||'Not detected'}</pre>{#if config?.targetError}<h3>Installation details</h3><pre>{config.targetError}</pre>{/if}<h3>Client routing</h3>{#each config?.catalog.targets??[] as target}<div class="um-target"><strong>Minecraft {target.id}</strong><span>{target.kind==='NATIVE'?'Native Windows':target.profile}</span></div>{/each}</details></section>
    {/if}
  </main>
</div>
