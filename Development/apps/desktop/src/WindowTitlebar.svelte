<script lang="ts">
  import {onMount} from 'svelte';
  import {getCurrentWindow} from '@tauri-apps/api/window';
  let maximized=false, closing=false, error='';
  async function action(operation:'minimize'|'toggleMaximize'|'close') {
    error='';
    try {
      if(operation==='close')closing=true;
      await getCurrentWindow()[operation]();
    } catch(reason) {error=String(reason);closing=false;}
  }
  onMount(()=>{
    let disposed=false,stop:(()=>void)|undefined;
    const window=getCurrentWindow();
    const update=()=>void window.isMaximized().then(value=>{if(!disposed)maximized=value;}).catch(reason=>{error=String(reason);});
    update();
    void window.onResized(update).then(unlisten=>{if(disposed)unlisten();else stop=unlisten;});
    return ()=>{disposed=true;stop?.();};
  });
</script>
<div class="lt-window-bar">
  <button class="lt-window-drag" data-tauri-drag-region aria-label="Move Lazy Tester window" on:dblclick={()=>action('toggleMaximize')}><span aria-hidden="true" data-tauri-drag-region>LT</span><span data-tauri-drag-region>Lazy Tester</span></button>
  <button class="lt-window-control" aria-label="Minimize" title="Minimize" on:click={()=>action('minimize')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10"/></svg></button>
  <button class="lt-window-control" aria-label={maximized?'Restore down':'Maximize'} title={maximized?'Restore down':'Maximize'} on:click={()=>action('toggleMaximize')}><svg viewBox="0 0 16 16" aria-hidden="true">{#if maximized}<path d="M5 5V3h8v8h-2M3 5h8v8H3Z"/>{:else}<path d="M3 3h10v10H3Z"/>{/if}</svg></button>
  <button class="lt-window-control lt-window-close" disabled={closing} aria-label="Close Lazy Tester" title="Close Lazy Tester — Minecraft stays open" on:click={()=>action('close')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3 3 10 10M13 3 3 13"/></svg></button>
</div>
{#if error}<p class="lt-alert" role="alert">{error}</p>{/if}
