<script lang="ts">
  import {onDestroy} from 'svelte';
  import type {ActivityView} from './app/presentation/agentActivity.js';
  export let view:ActivityView;
  let detailsElement: HTMLDetailsElement | undefined;
  let summaryElement: HTMLElement | undefined;
  function closeOnEscape(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || !detailsElement?.open ||
        !(event.target instanceof Node) || !detailsElement.contains(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    detailsElement.open = false;
    summaryElement?.focus({preventScroll: true});
  }
  function closeWhenFocusLeaves(event: FocusEvent): void {
    const next = event.relatedTarget;
    if (detailsElement?.open && next instanceof Node && !detailsElement.contains(next))
      detailsElement.open = false;
  }

  onDestroy(() => {
    // A short-lived controller report may disappear while its summary owns
    // keyboard focus. Restore focus only in that case, never from gameplay.
    const focused = document.activeElement;
    if (!(focused instanceof Node) || !detailsElement?.contains(focused)) return;
    const playNav = document.querySelector<HTMLButtonElement>(
      '.lt-topbar nav button[aria-current="page"]');
    if (playNav?.isConnected && !playNav.disabled)
      playNav.focus({preventScroll: true});
  });
</script>
<svelte:window on:keydown={closeOnEscape} />

<!-- Positioned out of normal layout: never changes viewport or rail size. -->
<details class="lt-agent-activity" name="lt-play-disclosures" bind:this={detailsElement} on:focusout={closeWhenFocusLeaves} aria-label="Local AI and Testing activity">
  <summary bind:this={summaryElement} aria-label={view.summary} title={view.summary}>{view.summary}</summary>
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
