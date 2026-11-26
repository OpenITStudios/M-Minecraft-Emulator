<script lang="ts">
  import {onDestroy,tick} from 'svelte';
  import type {ProfileSlot} from './app/bridge/lazyTester.js';
  import type {SessionHealthAction, SessionHealthIssue} from './app/presentation/sessionHealth.js';

  export let item: SessionHealthIssue;
  export let onAction: (slot: ProfileSlot, action: SessionHealthAction) => void;
  export let disabled = false;
  let noticeElement: HTMLDivElement | undefined;
  let statusElement: HTMLDivElement | undefined;
  let actionButton: HTMLButtonElement | undefined;

  // A focused Refresh may disable its own button while the native
  // observation is pending. Keep focus within the read-only, scrollable
  // message until the issue is actually removed. Select game retains the
  // original native presentation/focus behavior.
  function requestAction(): void {
    if (item.action === 'refresh' && document.activeElement === actionButton)
      statusElement?.focus({preventScroll:true});
    onAction(item.slot,item.action);
  }

  onDestroy(() => {
    if (item.action !== 'refresh') return;
    const previousFocus = document.activeElement;
    if (!(previousFocus instanceof Node) || !noticeElement?.contains(previousFocus)) return;
    const entry = noticeElement.closest('.lazytester-launcher-entry');
    if (!(entry instanceof HTMLElement)) return;
    // Wait for the original parent to apply the observed native status.
    // Do not focus an entry that went away during page navigation, a native
    // game focus switch, or a later explicit user focus change.
    void tick().then(() => {
      if (!entry.isConnected || !document.hasFocus()) return;
      const current = document.activeElement;
      if (current !== document.body && current !== previousFocus) return;
      const card = entry.querySelector<HTMLButtonElement>(':scope > .lazytester-launcher-card');
      if (card?.isConnected && !card.disabled)
        card.focus({preventScroll:true});
      else
        entry.focus({preventScroll:true});
    });
  });

  $: accessibleName = item.kind === 'preview'
    ? 'Minecraft - ' + item.slot + ' preview unavailable'
    : 'Minecraft - ' + item.slot + ' needs attention';
  // Match the global read-only Refresh action, not a competing "Check status".
  $: actionLabel = item.action === 'open' ? 'Select game' : 'Refresh';
  $: actionName = item.action === 'open' ? 'Select game' : 'Refresh status';
</script>

<div class="lazytester-health-notice" bind:this={noticeElement}>
  <div
    class={item.kind === 'preview' ? 'lazytester-error lazytester-preview-slot-issue' : 'lazytester-error lazytester-session-slot-issue'}
    role="status"
    bind:this={statusElement}
    aria-label={accessibleName}
    aria-describedby={'lazytester-health-guidance-'+item.slot}
    tabindex="0"
    title={item.title+' — '+item.guidance}
  >
    <strong>{item.title}</strong>
    <span id={'lazytester-health-guidance-'+item.slot}>{item.guidance}</span>
  </div>
  <button
    type="button"
    class="lazytester-health-action"
    bind:this={actionButton}
    disabled={disabled}
    aria-label={actionName + ' for Minecraft - ' + item.slot}
    aria-describedby={'lazytester-health-guidance-'+item.slot}
    on:click={requestAction}
  >{actionLabel}</button>
</div>

<style>
  .lazytester-health-notice { min-width: 0; font-size: 13px; line-height: 1.4; overflow-wrap: anywhere; }
  strong { display: block; font-weight: 600; }
  span { display: block; }
  .lazytester-health-action {
    /* A 28px pointer target with the same 28px base vertical footprint
       as the prior 24px control plus 4px margin. Keep five-slot DWM geometry. */
    display: inline-flex; align-items: center;
    box-sizing: border-box; min-height: 28px;
    margin-top: 0; padding: 3px 6px;
    border: 0; border-radius: 4px;
    background: transparent; color: var(--lazytester-accent);
    font: inherit; font-weight: 600; text-align: left; cursor: pointer;
  }
  .lazytester-health-action:hover:not(:disabled) {
    text-decoration: underline; background: var(--lazytester-selected-bg);
  }
  .lazytester-health-action:focus-visible { outline: 2px solid var(--lazytester-accent); outline-offset: 2px; border-radius: 2px; }
  .lazytester-health-action:disabled { color: var(--lazytester-muted); cursor: default; opacity: .65; }
  @media (forced-colors: active) {
    .lazytester-health-action { color: LinkText; border: 1px solid ButtonText; }
    .lazytester-health-action:disabled { color: GrayText; }
    .lazytester-health-action:focus-visible { outline-color: Highlight; }
  }
</style>
