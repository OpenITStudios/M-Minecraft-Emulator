<script lang="ts">
  import {onMount} from 'svelte';
  import {getCurrentWindow} from '@tauri-apps/api/window';
  import {listen} from '@tauri-apps/api/event';
  import {supportsManagedSessions, type NativeBackendSupport} from './app/bridge/lazyTester.js';
  import {userMessage, userFeedback, type UserMessageKey} from './app/presentation/sessionFeedback.js';
  // UI wording only; the native controller owns Minecraft exit.
  export let backendSupport: NativeBackendSupport | null = null;
  let closing=false;
  // Native window failures are private; the UI stores only authored keys.
  let error: UserMessageKey | null = null;
  async function requestAppExit() {
    error=null;
    try {
      // Native CloseRequested decides whether an exit is confirmed; a
      // cancelled dialog must not leave this control stuck on Closing.
      await getCurrentWindow().close();
    } catch {error='windowClose';closing=false;}
  }
  onMount(()=>{
    let disposed=false,stopError:(()=>void)|undefined,stopClosing:(()=>void)|undefined,stopRejected:(()=>void)|undefined;
    void listen<string>('lazy-tester-close-error',event=>{closing=false;error='closeIncomplete';}).then(unlisten=>{if(disposed)unlisten();else stopError=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    void listen<string>('lazy-tester-close-rejected',event=>{closing=false;error='closeRejected';}).then(unlisten=>{if(disposed)unlisten();else stopRejected=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    void listen<boolean>('lazy-tester-closing',event=>{closing=event.payload;}).then(unlisten=>{if(disposed)unlisten();else stopClosing=unlisten;}).catch(()=>{if(!disposed)error='windowStatus';});
    return ()=>{disposed=true;stopError?.();stopRejected?.();stopClosing?.();};
  });
</script>
<div class="lazytester-window-bar">
  <!-- Tauri owns native dragging; the exit button requests normal native shutdown. -->
  <div class="lazytester-window-drag" data-tauri-drag-region>
    <span class="lazytester-window-mark" aria-hidden="true" data-tauri-drag-region></span>
    <span data-tauri-drag-region>LazyTester</span>
  </div> 
  <button class="lazytester-window-control lazytester-window-close" disabled={closing} aria-label={supportsManagedSessions(backendSupport) ? 'Exit LazyTester and all managed Minecraft sessions' : 'Close LazyTester'} title={supportsManagedSessions(backendSupport) ? 'Exit LazyTester and all managed Minecraft sessions' : 'Close LazyTester'} on:click={requestAppExit}><svg viewBox="0 0 16 16" aria-hidden="true"><path d="m3 3 10 10M13 3 3 13"/></svg></button>
</div>
{#if closing}<div class="lazytester-alert" role="status">{supportsManagedSessions(backendSupport) ? 'Closing LazyTester… Waiting for managed games and pending work to finish.' : 'Closing LazyTester…'}</div>{/if}
{#if error}<div class="lazytester-alert" role="alert"><p>{userMessage(error)}</p><details class="lazytester-feedback-reference"><summary>Support reference</summary><p>{userFeedback(error).area} · {userFeedback(error).code}</p></details></div>{/if}
