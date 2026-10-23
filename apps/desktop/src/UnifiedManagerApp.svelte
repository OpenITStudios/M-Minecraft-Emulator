<script lang="ts">
  import {onMount,tick} from 'svelte';
  import OwnedManagerPanel from './OwnedManagerPanel.svelte';
  import NativeWorkspace from './NativeWorkspace.svelte';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import {nativeFeasibility,type NativeWorkspaceObservation} from './app/bridge/nativeFeasibilityApi.js';
  import type {ClientId} from './app/bridge/ownedProviderApi.js';
  import type {OwnedSnapshot} from './app/bridge/ownedProviderApi.js';
  import {managerConfiguration,operationIssue,type ManagerConfiguration} from './app/bridge/managerConfiguration.js';
  import {compareProcessContexts,type ContextComparison} from './app/bridge/processContext.js';
  let config:ManagerConfiguration|null=null,snapshot:OwnedSnapshot|null=null;
  let page:'clients'|'settings'='clients',busy=true,checked=false;
  let issue:ReturnType<typeof operationIssue>|null=null;
  let workspace:NativeWorkspace,panel:OwnedManagerPanel,selectedClient:ClientId|null=null,workspaceBusy=false;
  let tabStates:Record<ClientId,string>={"01":"Checking","02":"Checking","03":"Checking","04":"Checking"};
  let contextClient='02',contextResult:ContextComparison|null=null,contextError='';
  let nativeSession:NativeWorkspaceObservation|null=null,nativeError='',nativePending=false;
  $: if(!snapshot)tabStates={'01':nativeSession?.processes[0]?(nativeSession.processes[0].responding&&nativeSession.processes[0].windowHandle!=='0x0'?'Detected':'Starting'):nativePending?'Launch pending':nativeError?'Check needed':'Not opened','02':'Unavailable','03':'Unavailable','04':'Unavailable'};
  $: contextTarget=snapshot?.clients.find(slot=>slot.clientId===contextClient)?.processes;
  async function inspectContext(){
    if(busy||workspaceBusy)return;
    contextResult=null;contextError='';
    const native=snapshot?.nativeProcesses,other=contextTarget;
    if(native?.length!==1||other?.length!==1){contextError='Two unambiguous observed processes are required. New isolated launches remain disabled.';return;}
    busy=true;
    try{contextResult=await compareProcessContexts(native[0],other[0]);}
    catch{contextError='Windows identity could not be checked. The process may have exited, its identity may have changed, or access may be denied. No isolation result is established.';}
    finally{busy=false;}
  }
  async function navigate(next:'clients'|'settings'){
    if(busy||workspaceBusy)return;
    busy=true;
    try{if(next==='settings'){if(workspace)await workspace.suspend();else await workspaceSurface.suspend();page=next;}else{page=next;await tick();if(workspace)await workspace.resume();}}
    catch(error){issue=operationIssue(error);}finally{busy=false;}
  }
  // Native Minecraft 01 control path: no retired owned-provider inspection.
  async function inspectNative(){
    try{nativeSession=await nativeFeasibility.inspectNativeWorkspace();nativeError='';return nativeSession;}
    catch(error){nativeSession=null;nativeError=operationIssue(error).details;return null;}
  }
  async function refreshNative(){
    if(busy||workspaceBusy)return;
    busy=true;issue=null;
    try{
      const result=await inspectNative();
      if(result){
        nativePending=false;
        if(workspace)await workspace.reconcileNative(result);
      }
    }catch(error){issue=operationIssue(error);}
    finally{busy=false;}
  }
  async function activateNative(client:ClientId){
    if(client!=='01'||busy||workspaceBusy||!config?.detectedMinecraft)return;
    busy=true;issue=null;
    try{
      const observed=await inspectNative();
      if(!observed)return;
      const process=observed.processes[0];
      if(process){
        if(!process.responding||process.windowHandle==='0x0'){
          nativeError='Registered Minecraft has no responsive window yet. Refresh rather than launching a duplicate.';
          return;
        }
        await workspace.present('01',process);
        nativePending=false;return;
      }
      if(nativePending){
        nativeError='A previous launch request is pending. Refresh before retrying.';
        return;
      }
      // Explicit user click; Windows opens the uniquely registered official app.
      await nativeFeasibility.openOfficialMinecraft();
      nativePending=true;
      for(let attempt=0;attempt<6;attempt++){
        await new Promise<void>(resolve=>setTimeout(resolve,1400));
        const next=await inspectNative();
        if(!next)return;
        const candidate=next.processes[0];
        if(candidate?.responding&&candidate.windowHandle!=='0x0'){
          await workspace.present('01',candidate);
          nativePending=false;return;
        }
      }
      nativeError='Minecraft launch was requested but no responsive window was found. Refresh before another attempt.';
    }catch(error){issue=operationIssue(error);}
    finally{busy=false;}
  }
  async function prepare(){
    busy=true;checked=false;issue=null;contextResult=null;contextError='';
    try{config=await managerConfiguration.load();snapshot=config.observation;checked=!!snapshot;if(config.engineError&&snapshot)issue=operationIssue(config.engineError);if(config.detectedMinecraft)await inspectNative();await tick();if(workspace){try{if(snapshot)await workspace.reconcile(snapshot);else if(nativeSession)await workspace.reconcileNative(nativeSession);}catch(error){issue=operationIssue(error);}}}
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
    <p class="um-sidebar-foot">Windows developer test<br/>Independent engine pending</p>
  </aside>
  <main class="um-main">
    <header class="um-header"><div><p class="um-eyebrow">YOUR WORKSPACE</p><h1>{page==='clients'?'Minecraft clients':'Settings'}</h1><p>{page==='clients'?'Session workspace · independent engine pending.':'Read-only Minecraft and legacy session inspection; independent engine pending.'}</p></div><span class="um-engine">Legacy session snapshot · {checked?'Observed':busy?'Checking':'Unavailable'}</span></header>
    {#if issue}<div class="um-alert" role="alert"><strong>{issue.message}</strong><p>{issue.action}</p><details><summary>Diagnostics</summary><pre>{issue.details}</pre></details></div>{/if}
    <div class="um-client-view" hidden={page!=='clients'}>
      <p role="status">Sandboxie launch routes are retired. New sessions 02–04 are unavailable while the independent engine is developed. Existing sessions and data are preserved.</p>
      {#if config && snapshot}<div class="nw-layout"><NativeWorkspace bind:this={workspace} bind:working={workspaceBusy} sessionDirectory={config.settings.sessionDirectory} bind:activeClient={selectedClient} {tabStates} {busy} onRecovered={()=>issue=null} selectionBlocked={client=>panel?.actionBlockedReason(client)??null} onClose={(client,target)=>panel.closeClient(client,target)} onSelect={client=>panel.activateClient(client)}/><div class="nw-session-controls" aria-label="Session controls"><OwnedManagerPanel bind:this={panel} sessionDirectory={config.settings.sessionDirectory} bind:initialSnapshot={snapshot} bind:operationBusy={busy} bind:tabStates externalBusy={workspaceBusy} onPresent={(client,target)=>workspace.present(client,target)} onRelease={()=>workspace.suspend()} onResume={()=>workspace.resume()} onObserved={observation=>workspace.reconcile(observation)}/></div></div>
      {:else}
        {#if config && config.detectedMinecraft}
          <div class="nw-layout">
            <NativeWorkspace bind:this={workspace} bind:working={workspaceBusy} releaseOnly sessionDirectory="" bind:activeClient={selectedClient} {tabStates} {busy} onRecovered={()=>issue=null} onSelect={activateNative} selectionBlocked={client=>client==='01'?(nativeSession===null?'Refresh native status first':null):'Independent Minecraft 02–04 isolation is not implemented.'}/>
            <div class="nw-session-controls" aria-label="Native Minecraft controls">
              <div class="nw-session-tools"><p>Minecraft 01 uses the official Windows installation. No Sandboxie, owned fork or VM required. Sign-in remains inside Minecraft.</p></div>
              {#if nativeError}<p role="alert">{nativeError}</p>{/if}
              <p>{nativeSession?.processes[0]?'Registered Minecraft window candidate detected.':'No registered Minecraft process detected.'} This is not multi-account isolation.</p>
              <button class="cm-button" disabled={busy||workspaceBusy||nativeSession===null||nativePending} on:click={()=>activateNative('01')}>{busy?'Working…':nativeSession?.processes[0]?'Show Minecraft 01':'Open Minecraft 01'}</button>
              <button class="cm-button cm-secondary" disabled={busy||workspaceBusy} on:click={refreshNative}>Refresh native session</button>
              <p>Minecraft 02–04 remain disabled until a real independent provider exists. Return to Windows restores the game window without closing it.</p>
            </div>
          </div>
        {:else}
          <section class="um-setup"><h2>{busy?'Checking installation…':'Minecraft Education Desktop unavailable'}</h2><p>Install the official Desktop edition of Minecraft Education and refresh. Retired isolation engines are not required for Minecraft 01.</p><button class="cm-button" disabled={busy} on:click={prepare}>Check again</button><button class="cm-button cm-secondary" disabled={busy} on:click={()=>page='settings'}>View status</button></section>
        {/if}
      {/if}
    </div>
    {#if page==='settings'}
      <section class="um-settings"><details><summary>Independent engine diagnostics</summary>
        <p>Read-only comparison of Minecraft 01 and an existing observed process. This checks Windows identity only; it does not prove separate Minecraft accounts or launch a session.</p>
        <label>Compare with <select bind:value={contextClient} disabled={busy} on:change={()=>{contextResult=null;contextError='';}}><option value="02">Minecraft 02</option><option value="03">Minecraft 03</option><option value="04">Minecraft 04</option></select></label>
        <button class="cm-button cm-secondary" disabled={busy||workspaceBusy||snapshot?.nativeProcesses.length!==1||contextTarget?.length!==1} on:click={inspectContext}>Check Windows identity</button>
        {#if contextResult}<p role="status">Last explicit check: Windows user {contextResult.sameWindowsUser?'same':'different'}; configured profile directory {contextResult.sameProfileDirectory?'same':'different'}; Windows session {contextResult.sameWindowsSession?'same':'different'}. Processes may change after this check. Account isolation remains unproven.</p>{/if}
        {#if contextError}<p role="alert">{contextError}</p>{/if}
      </details></section>
      <section class="um-settings"><h2>Read-only status</h2><p>Detecting Minecraft and observing legacy sessions does not verify the independent engine, account isolation or gameplay readiness.</p>
        <div class="um-setting"><div><h3>Minecraft Education</h3><p>{busy?'Checking installation…':config?.targetError?'Installation needs attention. Repair Minecraft Education, then check again.':config?.detectedMinecraft?'Installed · detected automatically':'Not detected. Install Minecraft Education Desktop, then check again.'}</p></div></div>
        <div class="um-setting"><div><h3>Independent engine</h3><p>Not implemented · Client 02–04 Launch/Provision disabled.</p></div></div>
        <div class="um-setting"><div><h3>Native Minecraft 01</h3><p>{nativeSession?.processes[0]?'Registered game process candidate observed':nativeError?'Observation needs attention':'Independent of retired engine snapshots'}</p></div></div>
        <div class="um-setting"><div><h3>Legacy session snapshot</h3><p>{busy?'Checking read-only observation…':checked?'Observed · diagnostic only':'Unavailable · no engine readiness inferred'}</p></div></div>
        <button class="cm-button" disabled={busy} on:click={prepare}>{busy?'Checking…':'Check again'}</button>
      </section>
      <section class="um-settings"><details><summary>Diagnostics</summary><p>Windows developer test. Independent installation and production signing have separate acceptance requirements.</p><h3>Minecraft location</h3><pre>{config?.settings.minecraftExecutable||config?.detectedMinecraft||'Not detected'}</pre><h3>Legacy session storage location</h3><pre>{config?.settings.sessionDirectory||'Not detected'}</pre>{#if config?.targetError}<h3>Installation details</h3><pre>{config.targetError}</pre>{/if}<h3>Client routing</h3>{#each config?.catalog.targets??[] as target}<div class="um-target"><strong>Minecraft {target.id}</strong><span>{target.kind==='NATIVE'?'Native Windows':target.profile}</span></div>{/each}</details></section>
    {/if}
  </main>
</div>
