<script lang="ts">
  import type {ActivityView} from './app/presentation/agentActivity.js';
  export let view:ActivityView;
</script>

<!-- Positioned out of normal layout: never changes viewport or rail size. -->
<details class="lt-agent-activity" aria-label="Local AI and QA activity">
  <summary aria-label={view.summary}>{view.summary}</summary>
  <div class="lt-agent-activity-details">
    <strong>{view.urgent?'Attention required':'Agent activity'}</strong>
    {#each view.lines as item}
      <p>{item}</p>
    {/each}
    <small>{view.stale?
      'The original agent report has expired. Do not assume input or QA completed.' :
      'Reported by the local agent. Windows desktop control and map gameplay are not verified.'}</small>
  </div>
</details>
