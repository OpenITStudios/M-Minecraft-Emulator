<script lang="ts">
  import type {ActivityView} from './app/presentation/agentActivity.js';
  export let view:ActivityView;
</script>

<!-- Positioned out of normal layout: never changes viewport or rail size. -->
<details class="lt-agent-activity" aria-label="Local AI and Testing activity">
  <summary aria-label={view.summary}>{view.summary}</summary>
  <div class="lt-agent-activity-details">
    <strong>{view.urgent?'Attention required':'Activity overview'}</strong>
    {#each view.lines as item}
      <p>{item}</p>
    {/each}
    <small>{view.stale?
      'The latest status report has expired. AI input and testing outcomes are unconfirmed.' :
      'Reported by the local agent. PC control and Minecraft gameplay are not verified.'}</small>
  </div>
</details>
