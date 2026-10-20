<script lang="ts">
  import {onMount} from 'svelte';
  import OwnedManagerPanel from './OwnedManagerPanel.svelte';
  import {ownedProvider,type OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {nativeFeasibility} from './app/bridge/nativeFeasibilityApi.js';
  import {managerConfiguration,operationIssue,type ManagerConfiguration} from './app/bridge/managerConfiguration.js';
  let config:ManagerConfiguration|null=null,snapshot:OwnedSnapshot|null=null;
  let page:'clients'|'settings'='clients',busy=true,checked=false;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let feedback='';
  async function inspect(){
    if(busy||!config?.settings.sessionDirectory)return;
    busy=true;checked=false;issue=null;
    try{snapshot=await ownedProvider.execute(config.settings.sessionDirectory,'02','Inspect');checked=true;}
    catch(e){snapshot=null;issue=operationIssue(e);}finally{busy=false;}
  }
  onMount(()=>{void (async()=>{try{config=await managerConfiguration.load();snapshot=config.observation;checked=!!snapshot;if(config.engineError)issue=operationIssue(config.engineError);}catch(e){issue=operationIssue(e);}finally{busy=false;}})();});
  async function select(kind:'engine'|'minecraft'){
    if(busy||!config)return;
    busy=true;issue=null;feedback='';
    try{
      const path=kind==='engine'?await ownedProvider.selectSession():await nativeFeasibility.selectEducationExe();
      if(path){
        const settings={...config.settings,[kind==='engine'?'sessionDirectory':'minecraftExecutable']:path};
        config=await managerConfiguration.save(settings);snapshot=config.observation;checked=!!snapshot;if(config.engineError)issue=operationIssue(config.engineError);feedback='Location saved.';
      }
    }catch(e){issue=operationIssue(e);}finally{busy=false;}
    if(checked)feedback='Locations saved and engine verified for this session.';
  }
</script>

<svelte:head><title>Minecraft Instance Manager</title></svelte:head>
<div class="um-app">
  <aside class="um-sidebar"><div class="um-brand"><span class="um-logo">M</span><div>Minecraft<span>Instance Manager</span></div></div>
    <nav aria-label="Main navigation"><button class:active={page==='clients'} on:click={()=>page='clients'}>Clients <span>04</span></button><button class:active={page==='settings'} on:click={()=>page='settings'}>Settings</button></nav>
    <p class="um-sidebar-foot">Windows developer test<br/>One native · Three owned</p>
  </aside>
  <main class="um-main">
    <header class="um-header"><div><p class="um-eyebrow">YOUR WORKSPACE</p><h1>{page==='clients'?'Minecraft clients':'Settings'}</h1><p>{page==='clients'?'Four sessions. One place to set up, launch and manage.':'Choose the game and engine once. Their identity is checked before use.'}</p></div><span class="um-engine" class:ready={checked}>Engine · {busy?'Checking':checked?'Verified':'Needs setup'}</span></header>
    {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
    <div hidden={page!=='clients'}>
      {#if config && snapshot}<OwnedManagerPanel sessionDirectory={config.settings.sessionDirectory} bind:initialSnapshot={snapshot} bind:operationBusy={busy}/>
      {:else}
        <section class="um-setup"><h2>{busy?'Checking your engine…':'Connect your engine'}</h2><p>We could not connect to a verified owned engine automatically. Existing profiles and game data are preserved.</p><button class="cm-button" disabled={busy} on:click={()=>page='settings'}>Open Settings</button>{#if config?.settings.sessionDirectory}<button class="cm-button cm-secondary" disabled={busy} on:click={inspect}>Check again</button>{/if}</section>
        <section class="um-grid" aria-label="Minecraft clients">{#each config?.catalog.targets??[] as target}<article class="um-card"><p class="um-eyebrow">{target.kind==='NATIVE'?'NATIVE':'OWNED'}</p><h2>Minecraft {target.id}</h2><span class="um-status">Not configured</span><p>{target.profile??'Windows Desktop installation'}</p><button class="cm-button" disabled={busy} on:click={()=>page='settings'}>Set up</button></article>{/each}</section>
      {/if}
    </div>
    {#if page==='settings'}
      <section class="um-settings"><h2>Game & engine</h2>{#if config?.targetError}<div class="um-alert" role="alert"><strong>Minecraft needs attention</strong><details><summary>Diagnostics</summary><pre>{config.targetError}</pre></details></div>{/if}<div class="um-setting"><div><h3>Minecraft Education</h3><p>One registered Desktop installation is used by all four clients.</p><code>{config?.settings.minecraftExecutable||config?.detectedMinecraft||'No registered Desktop installation detected'}</code></div><button class="cm-button cm-secondary" disabled={busy||!config} on:click={()=>select('minecraft')}>Choose executable</button></div>
      <div class="um-setting"><div><h3>Owned engine</h3><p>Developer test package. Production installation is a separate acceptance step.</p><code>{config?.settings.sessionDirectory||'No engine selected'}</code></div><button class="cm-button cm-secondary" disabled={busy||!config} on:click={()=>select('engine')}>Choose engine</button></div>
      {#if feedback}<p role="status">{feedback}</p>{/if}<button class="cm-button" disabled={busy||!config?.settings.sessionDirectory} on:click={inspect}>Check configuration</button></section>
      <section class="um-settings"><h2>Minecraft targets</h2><p>Canonical catalog · engine/isolation-native/owned-runtime/minecraft-targets.json</p>{#each config?.catalog.targets??[] as target}<div class="um-target"><strong>Minecraft {target.id}</strong><span>{target.kind==='NATIVE'?'Native Windows':target.profile}</span></div>{/each}</section>
    {/if}
  </main>
</div>
