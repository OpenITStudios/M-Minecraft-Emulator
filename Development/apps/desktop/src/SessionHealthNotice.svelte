<script lang="ts">
  import type {ProfileSlot} from './app/bridge/lazyTester.js';
  import type {SessionHealthAction, SessionHealthIssue} from './app/presentation/sessionHealth.js';

  export let item: SessionHealthIssue;
  export let onAction: (slot: ProfileSlot, action: SessionHealthAction) => void;
  export let disabled = false;

  $: accessibleName = item.kind === 'preview'
    ? 'Minecraft - ' + item.slot + ' preview unavailable'
    : 'Minecraft - ' + item.slot + ' needs attention';
  $: actionLabel = item.action === 'open' ? 'Open game' : 'Check status';
</script>

<div class="lt-health-notice">
  <div
    class={item.kind === 'preview' ? 'lt-error lt-preview-slot-issue' : 'lt-error lt-session-slot-issue'}
    role="status"
    aria-label={accessibleName}
  >
    <strong>{item.title}</strong>
    <span>{item.guidance}</span>
  </div>
  <button
    type="button"
    class="lt-health-action"
    disabled={disabled}
    aria-label={actionLabel + ' for Minecraft - ' + item.slot}
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
