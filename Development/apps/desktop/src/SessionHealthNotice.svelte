<script lang="ts">
  import type {ProfileSlot} from './app/bridge/lazyTester.js';
  import type {SessionHealthAction, SessionHealthIssue} from './app/presentation/sessionHealth.js';

  export let item: SessionHealthIssue;
  export let onAction: (slot: ProfileSlot, action: SessionHealthAction) => void;
  export let disabled = false;

  $: accessibleName = item.kind === 'preview'
    ? 'Minecraft - ' + item.slot + ' preview unavailable'
    : 'Minecraft - ' + item.slot + ' needs attention';
  // Match the global read-only Refresh action, not a competing "Check status".
  $: actionLabel = item.action === 'open' ? 'Select game' : 'Refresh';
  $: actionName = item.action === 'open' ? 'Select game' : 'Refresh status';
</script>

<div class="lt-health-notice">
  <div
    class={item.kind === 'preview' ? 'lt-error lt-preview-slot-issue' : 'lt-error lt-session-slot-issue'}
    role="status"
    aria-label={accessibleName}
    aria-describedby={'lt-health-guidance-'+item.slot}
    tabindex="0"
    title={item.title+' — '+item.guidance}
  >
    <strong>{item.title}</strong>
    <span id={'lt-health-guidance-'+item.slot}>{item.guidance}</span>
  </div>
  <button
    type="button"
    class="lt-health-action"
    disabled={disabled}
    aria-label={actionName + ' for Minecraft - ' + item.slot}
    on:click={() => onAction(item.slot, item.action)}
  >{actionLabel}</button>
</div>

<style>
  .lt-health-notice { min-width: 0; font-size: 12px; line-height: 1.4; overflow-wrap: anywhere; }
  strong { display: block; font-weight: 600; }
  span { display: block; }
  .lt-health-action {
    display: inline-block; margin-top: 6px; padding: 3px 0;
    border: 0; background: transparent; color: var(--lt-accent);
    font: inherit; font-weight: 600; text-align: left; cursor: pointer;
  }
  .lt-health-action:hover:not(:disabled) { text-decoration: underline; }
  .lt-health-action:focus-visible { outline: 2px solid var(--lt-accent); outline-offset: 2px; border-radius: 2px; }
  .lt-health-action:disabled { color: var(--lt-muted); cursor: default; opacity: .65; }
</style>
