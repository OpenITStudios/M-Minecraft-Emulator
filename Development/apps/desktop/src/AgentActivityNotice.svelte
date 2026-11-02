<script lang="ts">
  import type {ActivityView} from './app/presentation/agentActivity.js';
  export let view:ActivityView;
</script>

<!-- Positioned out of normal layout: never changes viewport or rail size. -->
<details class="lt-agent-activity" aria-label="Local AI and Testing activity">
  <summary aria-label={view.summary} title={view.summary}>{view.summary}</summary>
  <div class="lt-agent-activity-details">
    <strong>{view.urgent?'Attention required':'Activity overview'}</strong>
    {#each view.lines as item}
      <p>{item}</p>
    {/each}
    {#each view.testingSummaries as result (result.slot)}
      <section class="lt-agent-testing-summary" aria-label={'Testing summary for '+result.slot}>
        <strong>Testing Summary</strong>
        <dl>
          <dt>Target</dt><dd>{result.slot}</dd>
          <dt>Probe Result</dt><dd>{result.outcome}</dd>
          {#if result.counts}
            <dt>Reported Checks</dt>
            <dd>{result.counts.expected} of {result.counts.expected} · {result.counts.pass} pass · {result.counts.fail} fail · {result.counts.info+result.counts.look} review</dd>
          {/if}
          <dt>Map Testing</dt><dd>Not tested</dd>
          <dt>Verification</dt><dd>Not verified</dd>
        </dl>
      </section>
    {/each}
    <small>{view.stale?
      'The latest status report has expired. AI input and testing outcomes are unconfirmed.' :
      'Reported by the local agent. PC control and Minecraft gameplay are not verified.'}</small>
  </div>
</details>
