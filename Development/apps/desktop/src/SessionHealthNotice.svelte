<script lang="ts">
  import type {SessionHealthIssue} from './app/presentation/sessionHealth.js';
  export let item: SessionHealthIssue;
  $: accessibleName = item.kind === 'preview'
    ? 'Minecraft - ' + item.slot + ' preview unavailable'
    : 'Minecraft - ' + item.slot + ' needs attention';
</script>

<small
  class={item.kind === 'preview' ? 'lt-error lt-preview-slot-issue' : 'lt-error lt-session-slot-issue'}
  role="status"
  aria-label={accessibleName}
>
  <strong>{item.title}</strong>
  <span>{item.guidance}</span>
</small>

<style>
  small {
    display: block;
    font-size: 12px;
    line-height: 1.4;
    overflow-wrap: anywhere;
  }
  strong { display: block; font-weight: 600; }
  span { display: block; }
</style>
