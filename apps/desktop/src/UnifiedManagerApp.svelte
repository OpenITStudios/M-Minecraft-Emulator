<script lang="ts">
  import {onMount} from 'svelte';
  import OwnedManagerPanel from './OwnedManagerPanel.svelte';
  import type {OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {managerConfiguration,operationIssue,type ManagerConfiguration} from './app/bridge/managerConfiguration.js';
  let config:ManagerConfiguration|null=null,snapshot:OwnedSnapshot|null=null;
  let page:'clients'|'settings'='clients',busy=true,checked=false;
  let issue:ReturnType<typeof operationIssue>|null=null;
  async function prepare(){
    busy=true;checked=false;issue=null;
    try{config=await managerConfiguration.load();snapshot=config.observation;checked=!!snapshot;if(config.engineError)issue=operationIssue(config.engineError);}
    catch(e){snapshot=null;issue=operationIssue(e);}finally{busy=false;}
  }
  onMount(()=>{void prepare();});
</script>

<svelte:head><title>Minecraft Instance Manager</title></svelte:head>
<div class="um-app">
  <aside class="um-sidebar"><div class="um-brand"><span class="um-logo">M</span><div>Minecraft<span>Instance Manager</span></div></div>
    <nav aria-label="Main navigation"><button class:active={page==='clients'} on:click={()=>page='clients'}>Clients <span>04</span></button><button class:active={page==='settings'} on:click={()=>page='settings'}>Settings</button></nav>
    <p class="um-sidebar-foot">Windows developer test<br/>One native · Three owned</p>
  </aside>
  <main class="um-main">
    <header class="um-header"><div><p class="um-eyebrow">YOUR WORKSPACE</p><h1>{page==='clients'?'Minecraft clients':'Settings'}</h1><p>{page==='clients'?'Four sessions. One place to launch and manage.':'Minecraft and the engine are detected automatically.'}</p></div><span class="um-engine" class:ready={checked}>Engine · {busy?'Checking':checked?'Verified':'Needs attention'}</span></header>
    {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
    <div hidden={page!=='clients'}>
      {#if config && snapshot}<OwnedManagerPanel sessionDirectory={config.settings.sessionDirectory} bind:initialSnapshot={snapshot} bind:operationBusy={busy}/>
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
