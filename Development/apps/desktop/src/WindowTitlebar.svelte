<script lang="ts">
  import {onMount} from 'svelte';
  import {getCurrentWindow} from '@tauri-apps/api/window';
  import {listen} from '@tauri-apps/api/event';
  import type {NativeBackendSupport} from './app/bridge/lazyTester.js';
  import {userMessage, userFeedback, type UserMessageKey} from './app/presentation/sessionFeedback.js';
  // UI wording only; the native controller owns Minecraft exit.
  export let backendSupport: NativeBackendSupport | null = null;
  let maximized=false, closing=false;
  // Native window failures are private; the UI stores only authored keys.
  let error: UserMessageKey | null = null;
  async function action(operation:'minimize'|'toggleMaximize'|'close') {
    error=null;
    try {
      // Native CloseRequested decides whether an exit is confirmed; a
      // cancelled dialog must not leave this control stuck on Closing.
      await getCurrentWindow()[operation]();
    } catch {error=operation==='close'?'windowClose':'windowControl';closing=false;}
  }
  onMount(()=>{
    let disposed=false,stop:(()=>void)|undefined,stopError:(()=>void)|undefined,stopClosing:(()=>void)|undefined,stopRejected:(()=>void)|undefined;
    const window=getCurrentWindow();
    const update=()=>void window.isMaximized().then(value=>{if(!disposed)maximized=value;}).catch(()=>{if(!disposed)error='windowStatus';});
    update();
    void window.onResized(update).then(unlisten=>{if(disposed)unlisten();else stop=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    void listen<string>('lazy-tester-close-error',event=>{closing=false;error='closeIncomplete';}).then(unlisten=>{if(disposed)unlisten();else stopError=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    void listen<string>('lazy-tester-close-rejected',event=>{closing=false;error='closeRejected';}).then(unlisten=>{if(disposed)unlisten();else stopRejected=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    void listen<boolean>('lazy-tester-closing',event=>{closing=event.payload;}).then(unlisten=>{if(disposed)unlisten();else stopClosing=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    return ()=>{disposed=true;stop?.();stopError?.();stopRejected?.();stopClosing?.();};
  });
</script>
<div class="lt-window-bar">
  <button class="lt-window-drag" data-tauri-drag-region aria-label="Move Lazy Tester window" on:dblclick={()=>action('toggleMaximize')}><span aria-hidden="true" data-tauri-drag-region>LT</span><span data-tauri-drag-region>Lazy Tester</span></button>
  <button class="lt-window-control" aria-label="Minimize" title="Minimize" on:click={()=>action('minimize')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 8h10"/></svg></button>
  <button class="lt-window-control" aria-label={maximized?'Restore down':'Maximize'} title={maximized?'Restore down':'Maximize'} on:click={()=>action('toggleMaximize')}><svg viewBox="0 0 16 16" aria-hidden="true">{#if maximized}<path d="M5 5V3h8v8h-2M3 5h8v8H3Z"/>{:else}<path d="M3 3h10v10H3Z"/>{/if}</svg></button>
  <button class="lt-window-control lt-window-close" disabled={closing} aria-label="Close Lazy Tester" title={backendSupport === 'WindowsNative' ? 'Exit Lazy Tester and all managed Minecraft sessions' : 'Close Lazy Tester'} on:click={()=>action('close')}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3 3 10 10M13 3 3 13"/></svg></button>
</div>
{#if closing}<div class="lt-alert" role="status">{backendSupport === 'WindowsNative' ? 'Closing Lazy Tester… Waiting for managed games and pending work to finish.' : 'Closing Lazy Tester…'}</div>{/if}
{#if error}<div class="lt-alert" role="alert"><p>{userMessage(error)}</p><details class="lt-feedback-reference"><summary>Support reference</summary><p>{userFeedback(error).area} · {userFeedback(error).code}</p></details></div>{/if}
