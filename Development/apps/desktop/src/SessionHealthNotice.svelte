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
    aria-describedby={'lt-health-guidance-'+item.slot}
    on:click={() => onAction(item.slot, item.action)}
  >{actionLabel}</button>
</div>

<style>
  .lt-health-notice { min-width: 0; font-size: 12px; line-height: 1.4; overflow-wrap: anywhere; }
  strong { display: block; font-weight: 600; }
  span { display: block; }
  .lt-health-action {
    /* A genuine 24px target. The 4px top margin offsets the former 6px
       margin to avoid increasing the fixed five-card rail's footprint. */
    display: inline-flex; align-items: center;
    box-sizing: border-box; min-height: 24px;
    margin-top: 4px; padding: 3px 6px;
    border: 0; border-radius: 4px;
    background: transparent; color: var(--lt-accent);
    font: inherit; font-weight: 600; text-align: left; cursor: pointer;
  }
  .lt-health-action:hover:not(:disabled) {
    text-decoration: underline; background: var(--lt-selected-bg);
  }
  .lt-health-action:focus-visible { outline: 2px solid var(--lt-accent); outline-offset: 2px; border-radius: 2px; }
  .lt-health-action:disabled { color: var(--lt-muted); cursor: default; opacity: .65; }
  @media (forced-colors: active) {
    .lt-health-action { color: LinkText; border: 1px solid ButtonText; }
    .lt-health-action:disabled { color: GrayText; }
    .lt-health-action:focus-visible { outline-color: Highlight; }
  }
</style>
