<script lang="ts">
  import {onMount} from 'svelte';
  import {getCurrentWindow} from '@tauri-apps/api/window';
  import {listen} from '@tauri-apps/api/event';
  let maximized=false, closing=false, error='';
  async function action(operation:'minimize'|'toggleMaximize'|'close') {
    error='';
    try {
      // Native CloseRequested decides whether an exit is confirmed; a
      // cancelled dialog must not leave this control stuck on Closing.
      await getCurrentWindow()[operation]();
    } catch(reason) {error=String(reason);closing=false;}
  }
  onMount(()=>{
    let disposed=false,stop:(()=>void)|undefined,stopError:(()=>void)|undefined,stopClosing:(()=>void)|undefined;
    const window=getCurrentWindow();
    const update=()=>void window.isMaximized().then(value=>{if(!disposed)maximized=value;}).catch(reason=>{error=String(reason);});
    update();
    void window.onResized(update).then(unlisten=>{if(disposed)unlisten();else stop=unlisten;}).catch(reason=>{error=String(reason);});
    void listen<string>('lazy-tester-close-error',event=>{closing=false;error='Exit did not finish. Review the remaining Minecraft sessions in Play.';}).then(unlisten=>{if(disposed)unlisten();else stopError=unlisten;}).catch(reason=>{error=String(reason);});
    void listen<boolean>('lazy-tester-closing',event=>{closing=event.payload;}).then(unlisten=>{if(disposed)unlisten();else stopClosing=unlisten;}).catch(reason=>{error=String(reason);});
    return ()=>{disposed=true;stop?.();stopError?.();stopClosing?.();};
  });
</script>
<div class="lt-window-bar">
  <button class="lt-window-drag" data-tauri-drag-region aria-label="Move Lazy Tester window" on:dblclick={()=>action('toggleMaximize')}><span aria-hidden="true" data-tauri-drag-region>LT</span><span data-tauri-drag-region>Lazy Tester</span></button>
  <button class="lt-window-control" aria-label="Minimize" title="Minimize" on:click={()=>action('minimize')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10"/></svg></button>
  <button class="lt-window-control" aria-label={maximized?'Restore down':'Maximize'} title={maximized?'Restore down':'Maximize'} on:click={()=>action('toggleMaximize')}><svg viewBox="0 0 16 16" aria-hidden="true">{#if maximized}<path d="M5 5V3h8v8h-2M3 5h8v8H3Z"/>{:else}<path d="M3 3h10v10H3Z"/>{/if}</svg></button>
  <button class="lt-window-control lt-window-close" disabled={closing} aria-label="Close Lazy Tester" title="Exit Lazy Tester and all managed Minecraft sessions" on:click={()=>action('close')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3 3 10 10M13 3 3 13"/></svg></button>
</div>
{#if closing}<div class="lt-alert" role="status">Closing Lazy Tester… Waiting for managed games and pending work to finish.</div>{/if}
{#if error}<p class="lt-alert" role="alert">{error}</p>{/if}
